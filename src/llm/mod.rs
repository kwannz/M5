pub mod router;
pub mod claude;
pub mod openrouter;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

pub use router::LlmRouter;
pub use claude::ClaudeClient;
pub use openrouter::OpenRouterClient;

/// Represents different LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Provider {
    Claude,
    OpenRouter,
    Offline,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Claude => write!(f, "claude"),
            Provider::OpenRouter => write!(f, "openrouter"),
            Provider::Offline => write!(f, "offline"),
        }
    }
}

/// Task types that determine routing strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Plan,
    Review,
    Status,
    Followup,
    Apply,
}

/// Configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub timeout_ms: u64,
}

/// Routing strategy for task types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub provider: Provider,
    pub temperature: f32,
}

/// Complete LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_provider: Provider,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub providers: std::collections::HashMap<Provider, ProviderConfig>,
    pub routing: std::collections::HashMap<TaskType, RouteConfig>,
    pub offline_mode: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        let mut providers = std::collections::HashMap::new();
        let mut routing = std::collections::HashMap::new();
        
        // Default provider configs (will be overridden by config file)
        providers.insert(Provider::Claude, ProviderConfig {
            api_key: "${ANTHROPIC_API_KEY}".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        });
        
        providers.insert(Provider::OpenRouter, ProviderConfig {
            api_key: "${OPENROUTER_API_KEY}".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            max_tokens: 4096,
            timeout_ms: 30000,
        });
        
        // Default routing strategies
        routing.insert(TaskType::Plan, RouteConfig {
            provider: Provider::Claude,
            temperature: 0.3,
        });
        
        routing.insert(TaskType::Review, RouteConfig {
            provider: Provider::Claude,
            temperature: 0.1,
        });
        
        routing.insert(TaskType::Status, RouteConfig {
            provider: Provider::OpenRouter,
            temperature: 0.0,
        });
        
        routing.insert(TaskType::Followup, RouteConfig {
            provider: Provider::Claude,
            temperature: 0.2,
        });
        
        routing.insert(TaskType::Apply, RouteConfig {
            provider: Provider::Claude,
            temperature: 0.0,
        });
        
        Self {
            default_provider: Provider::Claude,
            timeout_ms: 30000,
            max_retries: 3,
            providers,
            routing,
            offline_mode: false,
        }
    }
}

/// Request to LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub id: Uuid,
    pub task_type: TaskType,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Message roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Response from LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub id: Uuid,
    pub provider: Provider,
    pub model: String,
    pub content: String,
    pub usage: Usage,
    pub duration_ms: u64,
    pub cost_cents: Option<u32>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Provider-agnostic trait for LLM clients
#[async_trait::async_trait]
pub trait LlmProvider {
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse>;
    
    fn provider_name(&self) -> Provider;
    
    fn is_available(&self) -> bool;
}

/// Errors that can occur during LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Provider not available: {provider}")]
    ProviderNotAvailable { provider: Provider },
    
    #[error("Request failed: {message}")]
    RequestFailed { message: String },
    
    #[error("Rate limit exceeded for provider: {provider}")]
    RateLimited { provider: Provider },
    
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
    
    #[error("Offline mode active")]
    OfflineMode,
    
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
}

impl LlmRequest {
    pub fn new(task_type: TaskType, messages: Vec<Message>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            messages,
            temperature: None,
            max_tokens: None,
        }
    }
    
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content,
        }
    }
    
    pub fn assistant(content: String) -> Self {
        Self {
            role: MessageRole::Assistant,
            content,
        }
    }
    
    pub fn system(content: String) -> Self {
        Self {
            role: MessageRole::System,
            content,
        }
    }
}