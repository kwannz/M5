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
pub struct OpenRouterClient {
    client: Client,
    config: ProviderConfig,
}

#[derive(Debug, Serialize)]
struct OpenRouterApiRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<OpenRouterMessage>,
}

#[derive(Debug, Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterApiResponse {
    id: String,
    model: String,
    usage: OpenRouterUsage,
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterErrorResponse {
    error: OpenRouterError,
}

#[derive(Debug, Deserialize)]
struct OpenRouterError {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

impl OpenRouterClient {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    fn build_request(&self, request: &LlmRequest) -> OpenRouterApiRequest {
        let messages = request.messages.iter().map(|msg| {
            OpenRouterMessage {
                role: self.map_message_role(&msg.role),
                content: msg.content.clone(),
            }
        }).collect();
        
        OpenRouterApiRequest {
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
            MessageRole::System => "system".to_string(),
        }
    }
    
    async fn make_api_request(&self, openrouter_request: &OpenRouterApiRequest) -> Result<OpenRouterApiResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("HTTP-Referer", "https://github.com/anthropics/deskagent")
            .header("X-Title", "DeskAgent")
            .json(openrouter_request)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response: {}", e))?;
        
        if status.is_success() {
            serde_json::from_str::<OpenRouterApiResponse>(&response_text)
                .map_err(|e| anyhow!("Failed to parse OpenRouter response: {}", e))
        } else {
            // Try to parse as error response
            if let Ok(error_response) = serde_json::from_str::<OpenRouterErrorResponse>(&response_text) {
                if let Some(code) = &error_response.error.code {
                    if code == "rate_limit_exceeded" || status.as_u16() == 429 {
                        return Err(LlmError::RateLimited { provider: Provider::OpenRouter }.into());
                    }
                }
                Err(LlmError::RequestFailed { 
                    message: format!("OpenRouter API error: {}", error_response.error.message) 
                }.into())
            } else {
                Err(anyhow!("OpenRouter API request failed with status {}: {}", status, response_text))
            }
        }
    }
    
    fn parse_response(&self, request_id: uuid::Uuid, openrouter_response: OpenRouterApiResponse, duration_ms: u64) -> Result<LlmResponse> {
        let choice = openrouter_response.choices.into_iter().next()
            .ok_or_else(|| anyhow!("No choices in OpenRouter response"))?;
        
        let usage = Usage {
            prompt_tokens: openrouter_response.usage.prompt_tokens,
            completion_tokens: openrouter_response.usage.completion_tokens,
            total_tokens: openrouter_response.usage.total_tokens,
        };
        
        // Estimate cost based on model - varies by model on OpenRouter
        // Using a conservative estimate of $0.0005/1K tokens for input, $0.002/1K tokens for output
        let cost_cents = Some(
            (usage.prompt_tokens as f64 * 0.00005 + usage.completion_tokens as f64 * 0.0002) as u32
        );
        
        let response = LlmResponse {
            id: request_id,
            provider: Provider::OpenRouter,
            model: openrouter_response.model,
            content: choice.message.content,
            usage,
            duration_ms,
            cost_cents,
        };
        
        Ok(response)
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenRouterClient {
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let start_time = Instant::now();
        
        let openrouter_request = self.build_request(request);
        let openrouter_response = self.make_api_request(&openrouter_request).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let response = self.parse_response(request.id, openrouter_response, duration_ms)?;
        
        Ok(response)
    }
    
    fn provider_name(&self) -> Provider {
        Provider::OpenRouter
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
    fn test_openrouter_client_creation() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = OpenRouterClient::new(config);
        assert_eq!(client.provider_name(), Provider::OpenRouter);
        assert!(client.is_available());
    }
    
    #[test]
    fn test_message_role_mapping() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = OpenRouterClient::new(config);
        
        assert_eq!(client.map_message_role(&MessageRole::User), "user");
        assert_eq!(client.map_message_role(&MessageRole::Assistant), "assistant");
        assert_eq!(client.map_message_role(&MessageRole::System), "system");
    }
    
    #[test]
    fn test_build_request() {
        let config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = OpenRouterClient::new(config);
        
        let messages = vec![
            Message::system("You are a helpful assistant".to_string()),
            Message::user("Hello".to_string()),
        ];
        
        let request = LlmRequest::new(TaskType::Review, messages)
            .with_temperature(0.1)
            .with_max_tokens(1000);
        
        let openrouter_request = client.build_request(&request);
        
        assert_eq!(openrouter_request.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(openrouter_request.max_tokens, 1000);
        assert_eq!(openrouter_request.temperature, 0.1);
        assert_eq!(openrouter_request.messages.len(), 2);
        assert_eq!(openrouter_request.messages[0].role, "system");
        assert_eq!(openrouter_request.messages[1].role, "user");
    }
    
    #[test]
    fn test_availability_check() {
        let mut config = ProviderConfig {
            api_key: "test-key".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        };
        
        let client = OpenRouterClient::new(config.clone());
        assert!(client.is_available());
        
        // Test with empty API key
        config.api_key = "".to_string();
        let client = OpenRouterClient::new(config.clone());
        assert!(!client.is_available());
        
        // Test with environment variable placeholder
        config.api_key = "${OPENROUTER_API_KEY}".to_string();
        let client = OpenRouterClient::new(config);
        assert!(!client.is_available());
    }
}