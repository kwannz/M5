use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use uuid::Uuid;

use super::{
    LlmProvider, LlmRequest, LlmResponse, Provider, LlmConfig, TaskType, 
    ClaudeClient, OpenRouterClient, LlmError, RouteConfig
};

pub struct LlmRouter {
    config: LlmConfig,
    providers: HashMap<Provider, Arc<dyn LlmProvider + Send + Sync>>,
    log_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLog {
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
    pub task_type: TaskType,
    pub attempted_provider: Provider,
    pub final_provider: Provider,
    pub success: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub cost_cents: Option<u32>,
    pub tokens_used: u32,
}

impl LlmRouter {
    pub async fn new(config: LlmConfig, log_dir: &str) -> Result<Self> {
        // Create logging directory
        create_dir_all(log_dir).await?;
        let log_file_path = format!("{}/log.jsonl", log_dir);
        
        // Initialize providers
        let mut providers: HashMap<Provider, Arc<dyn LlmProvider + Send + Sync>> = HashMap::new();
        
        // Add Claude provider if configured
        if let Some(claude_config) = config.providers.get(&Provider::Claude) {
            let client = ClaudeClient::new(claude_config.clone());
            providers.insert(Provider::Claude, Arc::new(client));
        }
        
        // Add OpenRouter provider if configured
        if let Some(openrouter_config) = config.providers.get(&Provider::OpenRouter) {
            let client = OpenRouterClient::new(openrouter_config.clone());
            providers.insert(Provider::OpenRouter, Arc::new(client));
        }
        
        Ok(Self {
            config,
            providers,
            log_file_path,
        })
    }
    
    /// Route and execute an LLM request with retry logic and fallback
    pub async fn generate(&self, mut request: LlmRequest) -> Result<LlmResponse> {
        if self.config.offline_mode {
            return Err(LlmError::OfflineMode.into());
        }
        
        let start_time = std::time::Instant::now();
        let route_config = self.get_route_config(&request.task_type);
        
        // Apply routing configuration
        if request.temperature.is_none() {
            request = request.with_temperature(route_config.temperature);
        }
        
        let mut attempted_providers = Vec::new();
        let mut last_error = None;
        
        // Try primary provider
        let primary_provider = route_config.provider.clone();
        attempted_providers.push(primary_provider.clone());
        
        match self.try_provider(&primary_provider, &request).await {
            Ok(response) => {
                self.log_request(&request, &primary_provider, &primary_provider, true, 
                               start_time.elapsed().as_millis() as u64, None, 0, response.cost_cents, response.usage.total_tokens).await;
                return Ok(response);
            }
            Err(e) => {
                log::warn!("Primary provider {} failed: {}", primary_provider, e);
                last_error = Some(e);
            }
        }
        
        // Try fallback providers with retry logic
        for retry_count in 0..self.config.max_retries {
            for (provider, provider_client) in &self.providers {
                // Skip already attempted providers
                if attempted_providers.contains(provider) {
                    continue;
                }
                
                if !provider_client.is_available() {
                    continue;
                }
                
                attempted_providers.push(provider.clone());
                
                match self.try_provider(provider, &request).await {
                    Ok(response) => {
                        self.log_request(&request, &primary_provider, provider, true, 
                                       start_time.elapsed().as_millis() as u64, None, retry_count + 1, 
                                       response.cost_cents, response.usage.total_tokens).await;
                        return Ok(response);
                    }
                    Err(e) => {
                        log::warn!("Fallback provider {} failed (retry {}): {}", provider, retry_count, e);
                        last_error = Some(e);
                        
                        // Wait before retrying (exponential backoff)
                        let delay_ms = 500 * 2_u64.pow(retry_count);
                        sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
        
        // All providers failed
        let error_message = last_error.as_ref()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "All providers failed".to_string());
        
        self.log_request(&request, &primary_provider, &Provider::Offline, false, 
                       start_time.elapsed().as_millis() as u64, Some(error_message.clone()), 
                       self.config.max_retries, None, 0).await;
        
        Err(LlmError::MaxRetriesExceeded.into())
    }
    
    async fn try_provider(&self, provider: &Provider, request: &LlmRequest) -> Result<LlmResponse> {
        let provider_client = self.providers.get(provider)
            .ok_or_else(|| LlmError::ProviderNotAvailable { provider: provider.clone() })?;
        
        if !provider_client.is_available() {
            return Err(LlmError::ProviderNotAvailable { provider: provider.clone() }.into());
        }
        
        provider_client.generate(request).await
    }
    
    fn get_route_config(&self, task_type: &TaskType) -> RouteConfig {
        self.config.routing.get(task_type)
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to default routing
                RouteConfig {
                    provider: self.config.default_provider.clone(),
                    temperature: 0.7,
                }
            })
    }
    
    async fn log_request(&self, request: &LlmRequest, attempted_provider: &Provider, 
                        final_provider: &Provider, success: bool, duration_ms: u64, 
                        error_message: Option<String>, retry_count: u32, cost_cents: Option<u32>, tokens_used: u32) {
        let log_entry = RouteLog {
            timestamp: Utc::now(),
            request_id: request.id,
            task_type: request.task_type.clone(),
            attempted_provider: attempted_provider.clone(),
            final_provider: final_provider.clone(),
            success,
            duration_ms,
            error_message,
            retry_count,
            cost_cents,
            tokens_used,
        };
        
        if let Err(e) = self.write_log_entry(&log_entry).await {
            log::error!("Failed to write routing log: {}", e);
        }
    }
    
    async fn write_log_entry(&self, log_entry: &RouteLog) -> Result<()> {
        let log_line = serde_json::to_string(log_entry)?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .await?;
        
        file.write_all(format!("{}\n", log_line).as_bytes()).await?;
        file.flush().await?;
        
        Ok(())
    }
    
    /// Check which providers are currently available
    pub fn get_available_providers(&self) -> Vec<Provider> {
        self.providers.iter()
            .filter_map(|(provider, client)| {
                if client.is_available() {
                    Some(provider.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get routing statistics from log
    pub async fn get_routing_stats(&self) -> Result<RoutingStats> {
        // This would read the log file and compute statistics
        // For now, return empty stats
        Ok(RoutingStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            provider_usage: HashMap::new(),
            average_duration_ms: 0,
            total_cost_cents: 0,
        })
    }
    
    /// Enable or disable offline mode
    pub fn set_offline_mode(&mut self, offline: bool) {
        self.config.offline_mode = offline;
    }
    
    /// Check if router is in offline mode
    pub fn is_offline_mode(&self) -> bool {
        self.config.offline_mode
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub provider_usage: HashMap<Provider, u32>,
    pub average_duration_ms: u64,
    pub total_cost_cents: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Message, ProviderConfig};
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_router_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = LlmConfig::default();
        
        let router = LlmRouter::new(config, temp_dir.path().to_str().unwrap()).await;
        assert!(router.is_ok());
        
        let router = router.unwrap();
        assert!(!router.is_offline_mode());
    }
    
    #[tokio::test]
    async fn test_offline_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = LlmConfig::default();
        config.offline_mode = true;
        
        let router = LlmRouter::new(config, temp_dir.path().to_str().unwrap()).await.unwrap();
        
        let messages = vec![Message::user("Hello".to_string())];
        let request = LlmRequest::new(TaskType::Plan, messages);
        
        let result = router.generate(request).await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_string = e.to_string();
            assert!(error_string.contains("Offline mode active"));
        }
    }
    
    #[tokio::test]
    async fn test_route_config_lookup() {
        let temp_dir = TempDir::new().unwrap();
        let config = LlmConfig::default();
        
        let router = LlmRouter::new(config, temp_dir.path().to_str().unwrap()).await.unwrap();
        
        let plan_config = router.get_route_config(&TaskType::Plan);
        assert_eq!(plan_config.provider, Provider::Claude);
        assert_eq!(plan_config.temperature, 0.3);
        
        let review_config = router.get_route_config(&TaskType::Review);
        assert_eq!(review_config.provider, Provider::Claude);
        assert_eq!(review_config.temperature, 0.1);
        
        let status_config = router.get_route_config(&TaskType::Status);
        assert_eq!(status_config.provider, Provider::OpenRouter);
        assert_eq!(status_config.temperature, 0.0);
    }
    
    #[test]
    fn test_available_providers() {
        // This test would require actual API keys, so we'll just test the structure
        let temp_dir = TempDir::new().unwrap();
        let config = LlmConfig::default();
        
        // Without valid API keys, no providers should be available
        assert!(!config.providers[&Provider::Claude].api_key.starts_with("sk-"));
        assert!(!config.providers[&Provider::OpenRouter].api_key.starts_with("sk-"));
    }
    
    #[tokio::test]
    async fn test_log_entry_serialization() {
        let log_entry = RouteLog {
            timestamp: Utc::now(),
            request_id: Uuid::new_v4(),
            task_type: TaskType::Plan,
            attempted_provider: Provider::Claude,
            final_provider: Provider::OpenRouter,
            success: true,
            duration_ms: 1500,
            error_message: None,
            retry_count: 1,
            cost_cents: Some(15),
            tokens_used: 1000,
        };
        
        let serialized = serde_json::to_string(&log_entry);
        assert!(serialized.is_ok());
        
        let deserialized: Result<RouteLog, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}