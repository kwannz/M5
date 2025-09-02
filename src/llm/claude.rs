use anyhow::{Result, anyhow};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::time::Duration;

use super::{
    LlmProvider, LlmRequest, LlmResponse, Message, MessageRole, Provider, 
    ProviderConfig, Usage, LlmError
};

#[derive(Debug)]
pub struct ClaudeClient {
    client: Client,
    config: ProviderConfig,
}

#[derive(Debug, Serialize)]
struct ClaudeApiRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeApiResponse {
    id: String,
    model: String,
    usage: ClaudeUsage,
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorResponse {
    error: ClaudeError,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl ClaudeClient {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    fn build_request(&self, request: &LlmRequest) -> ClaudeApiRequest {
        let messages = request.messages.iter().map(|msg| {
            ClaudeMessage {
                role: self.map_message_role(&msg.role),
                content: msg.content.clone(),
            }
        }).collect();
        
        ClaudeApiRequest {
            model: self.config.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            temperature: request.temperature.unwrap_or(0.7),
            messages,
        }
    }
    
    fn map_message_role(&self, role: &MessageRole) -> String {
        match role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::System => "user".to_string(), // Claude doesn't have system role, map to user
        }
    }
    
    async fn make_api_request(&self, claude_request: &ClaudeApiRequest) -> Result<ClaudeApiResponse> {
        let url = format!("{}/messages", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(claude_request)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response: {}", e))?;
        
        if status.is_success() {
            serde_json::from_str::<ClaudeApiResponse>(&response_text)
                .map_err(|e| anyhow!("Failed to parse Claude response: {}", e))
        } else {
            // Try to parse as error response
            if let Ok(error_response) = serde_json::from_str::<ClaudeErrorResponse>(&response_text) {
                match error_response.error.error_type.as_str() {
                    "rate_limit_error" => Err(LlmError::RateLimited { provider: Provider::Claude }.into()),
                    _ => Err(LlmError::RequestFailed { 
                        message: format!("Claude API error: {}", error_response.error.message) 
                    }.into()),
                }
            } else {
                Err(anyhow!("Claude API request failed with status {}: {}", status, response_text))
            }
        }
    }
    
    fn parse_response(&self, request_id: uuid::Uuid, claude_response: ClaudeApiResponse, duration_ms: u64) -> LlmResponse {
        let content = claude_response.content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n");
        
        let usage = Usage {
            prompt_tokens: claude_response.usage.input_tokens,
            completion_tokens: claude_response.usage.output_tokens,
            total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
        };
        
        // Estimate cost based on Claude 3.5 Sonnet pricing
        // Input: $3/1M tokens, Output: $15/1M tokens
        let cost_cents = Some(
            (usage.prompt_tokens as f64 * 0.0003 + usage.completion_tokens as f64 * 0.0015) as u32
        );
        
        LlmResponse {
            id: request_id,
            provider: Provider::Claude,
            model: claude_response.model,
            content,
            usage,
            duration_ms,
            cost_cents,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for ClaudeClient {
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let start_time = Instant::now();
        
        let claude_request = self.build_request(request);
        let claude_response = self.make_api_request(&claude_request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let response = self.parse_response(request.id, claude_response, duration_ms);
        
        Ok(response)
    }
    
    fn provider_name(&self) -> Provider {
        Provider::Claude
    }
    
    fn is_available(&self) -> bool {
        !self.config.api_key.is_empty() && !self.config.api_key.starts_with('$')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{TaskType, Message, MessageRole};
    
    #[test]
    fn test_claude_client_creation() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = ClaudeClient::new(config);
        assert_eq!(client.provider_name(), Provider::Claude);
        assert!(client.is_available());
    }
    
    #[test]
    fn test_message_role_mapping() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = ClaudeClient::new(config);
        
        assert_eq!(client.map_message_role(&MessageRole::User), "user");
        assert_eq!(client.map_message_role(&MessageRole::Assistant), "assistant");
        assert_eq!(client.map_message_role(&MessageRole::System), "user"); // Maps to user
    }
    
    #[test]
    fn test_build_request() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = ClaudeClient::new(config);
        
        let messages = vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi there!".to_string()),
        ];
        
        let request = LlmRequest::new(TaskType::Plan, messages)
            .with_temperature(0.3)
            .with_max_tokens(2000);
        
        let claude_request = client.build_request(&request);
        
        assert_eq!(claude_request.model, "claude-3-5-sonnet-20241022");
        assert_eq!(claude_request.max_tokens, 2000);
        assert_eq!(claude_request.temperature, 0.3);
        assert_eq!(claude_request.messages.len(), 2);
        assert_eq!(claude_request.messages[0].role, "user");
        assert_eq!(claude_request.messages[0].content, "Hello");
    }
    
    #[test]
    fn test_availability_check() {
        let mut config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = ClaudeClient::new(config.clone());
        assert!(client.is_available());
        
        // Test with empty API key
        config.api_key = "".to_string();
        let client = ClaudeClient::new(config.clone());
        assert!(!client.is_available());
        
        // Test with environment variable placeholder
        config.api_key = "${ANTHROPIC_API_KEY}".to_string();
        let client = ClaudeClient::new(config);
        assert!(!client.is_available());
    }
}