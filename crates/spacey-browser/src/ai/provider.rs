//! AI Provider abstraction for BYOK (Bring Your Own Key) support.
//!
//! This module provides a unified interface for interacting with different AI providers:
//! - Local: Phi-3 model running on device
//! - Claude: Anthropic's API
//! - OpenAI: ChatGPT API

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Supported AI providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderType {
    #[default]
    Local,
    Claude,
    OpenAI,
}

impl std::fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProviderType::Local => write!(f, "local"),
            AiProviderType::Claude => write!(f, "claude"),
            AiProviderType::OpenAI => write!(f, "openai"),
        }
    }
}

/// Configuration for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub provider: AiProviderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            provider: AiProviderType::Local,
            api_key: None,
            model: Some("phi-3-mini-4k".to_string()),
            base_url: None,
        }
    }
}

/// Error types for AI provider operations
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API key required for {0}")]
    ApiKeyRequired(AiProviderType),
    
    #[error("Invalid API key format")]
    InvalidApiKeyFormat,
    
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    
    #[error("Model not loaded")]
    ModelNotLoaded,
    
    #[error("Provider not supported: {0}")]
    NotSupported(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("JSON parse error: {0}")]
    JsonError(String),
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Trait for AI providers
pub trait AiProvider: Send + Sync {
    /// Generate a response to the given messages
    fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, ProviderError>;
    
    /// Check if the provider is ready to use
    fn is_ready(&self) -> bool;
    
    /// Get the provider type
    fn provider_type(&self) -> AiProviderType;
}

/// Claude (Anthropic) API provider
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Result<Self, ProviderError> {
        if !api_key.starts_with("sk-ant-") {
            return Err(ProviderError::InvalidApiKeyFormat);
        }
        
        Ok(Self {
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }
}

impl AiProvider for ClaudeProvider {
    fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, ProviderError> {
        // Convert messages to Claude format
        let claude_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "user", // Should be filtered
                    },
                    "content": m.content
                })
            })
            .collect();
        
        // Extract system message if present
        let system_message = messages
            .iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.clone());
        
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "messages": claude_messages
        });
        
        if let Some(system) = system_message {
            body["system"] = serde_json::Value::String(system);
        }
        
        // Make the API request
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(ProviderError::ApiRequestFailed(error_text));
        }
        
        let json: serde_json::Value = response
            .json()
            .map_err(|e| ProviderError::JsonError(e.to_string()))?;
        
        // Extract content from Claude response
        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProviderError::JsonError("Missing content in response".to_string()))
    }
    
    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
    
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Claude
    }
}

/// OpenAI (ChatGPT) API provider
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Result<Self, ProviderError> {
        if !api_key.starts_with("sk-") {
            return Err(ProviderError::InvalidApiKeyFormat);
        }
        
        Ok(Self {
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }
}

impl AiProvider for OpenAiProvider {
    fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, ProviderError> {
        // Convert messages to OpenAI format
        let openai_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        MessageRole::System => "system",
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                    },
                    "content": m.content
                })
            })
            .collect();
        
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "messages": openai_messages
        });
        
        // Make the API request
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(ProviderError::ApiRequestFailed(error_text));
        }
        
        let json: serde_json::Value = response
            .json()
            .map_err(|e| ProviderError::JsonError(e.to_string()))?;
        
        // Extract content from OpenAI response
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProviderError::JsonError("Missing content in response".to_string()))
    }
    
    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
    
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::OpenAI
    }
}

/// Local Phi-3 model provider (wrapper around existing implementation)
pub struct LocalProvider {
    model: Option<super::model::Phi3Model>,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self { model: None }
    }
    
    pub fn load_model(&mut self, config: super::model::ModelConfig) -> Result<(), ProviderError> {
        self.model = Some(
            super::model::Phi3Model::new(config)
                .map_err(|e| ProviderError::ApiRequestFailed(format!("{:?}", e)))?
        );
        Ok(())
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for LocalProvider {
    fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, ProviderError> {
        let model = self.model.as_ref().ok_or(ProviderError::ModelNotLoaded)?;
        
        // Convert messages to a single prompt for Phi-3
        let prompt = messages
            .iter()
            .map(|m| {
                match m.role {
                    MessageRole::System => format!("<|system|>\n{}<|end|>", m.content),
                    MessageRole::User => format!("<|user|>\n{}<|end|>", m.content),
                    MessageRole::Assistant => format!("<|assistant|>\n{}<|end|>", m.content),
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let full_prompt = format!("{}\n<|assistant|>\n", prompt);
        
        model
            .generate(&full_prompt, max_tokens)
            .map_err(|e| ProviderError::ApiRequestFailed(format!("{:?}", e)))
    }
    
    fn is_ready(&self) -> bool {
        self.model.as_ref().map_or(false, |m| m.is_loaded())
    }
    
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Local
    }
}

/// Provider manager that handles switching between providers
pub struct ProviderManager {
    config: Arc<RwLock<AiProviderConfig>>,
    local: Arc<RwLock<LocalProvider>>,
    claude: Arc<RwLock<Option<ClaudeProvider>>>,
    openai: Arc<RwLock<Option<OpenAiProvider>>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AiProviderConfig::default())),
            local: Arc::new(RwLock::new(LocalProvider::new())),
            claude: Arc::new(RwLock::new(None)),
            openai: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> AiProviderConfig {
        self.config.read().clone()
    }
    
    /// Set the configuration
    pub fn set_config(&self, config: AiProviderConfig) -> Result<(), ProviderError> {
        // Validate and set up the provider
        match config.provider {
            AiProviderType::Local => {
                // Local doesn't need API key
            }
            AiProviderType::Claude => {
                let api_key = config.api_key.as_ref()
                    .ok_or(ProviderError::ApiKeyRequired(AiProviderType::Claude))?;
                let provider = ClaudeProvider::new(api_key.clone(), config.model.clone())?;
                *self.claude.write() = Some(provider);
            }
            AiProviderType::OpenAI => {
                let api_key = config.api_key.as_ref()
                    .ok_or(ProviderError::ApiKeyRequired(AiProviderType::OpenAI))?;
                let provider = OpenAiProvider::new(api_key.clone(), config.model.clone())?;
                *self.openai.write() = Some(provider);
            }
        }
        
        *self.config.write() = config;
        Ok(())
    }
    
    /// Generate a response using the current provider
    pub fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, ProviderError> {
        let config = self.config.read();
        
        match config.provider {
            AiProviderType::Local => {
                self.local.read().generate(messages, max_tokens)
            }
            AiProviderType::Claude => {
                self.claude.read()
                    .as_ref()
                    .ok_or(ProviderError::ApiKeyRequired(AiProviderType::Claude))?
                    .generate(messages, max_tokens)
            }
            AiProviderType::OpenAI => {
                self.openai.read()
                    .as_ref()
                    .ok_or(ProviderError::ApiKeyRequired(AiProviderType::OpenAI))?
                    .generate(messages, max_tokens)
            }
        }
    }
    
    /// Load the local Phi-3 model
    pub fn load_local_model(&self, config: super::model::ModelConfig) -> Result<(), ProviderError> {
        self.local.write().load_model(config)
    }
    
    /// Check if the current provider is ready
    pub fn is_ready(&self) -> bool {
        let config = self.config.read();
        
        match config.provider {
            AiProviderType::Local => self.local.read().is_ready(),
            AiProviderType::Claude => {
                self.claude.read().as_ref().map_or(false, |p| p.is_ready())
            }
            AiProviderType::OpenAI => {
                self.openai.read().as_ref().map_or(false, |p| p.is_ready())
            }
        }
    }
    
    /// Get the current provider type
    pub fn current_provider(&self) -> AiProviderType {
        self.config.read().provider
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_config_default() {
        let config = AiProviderConfig::default();
        assert_eq!(config.provider, AiProviderType::Local);
        assert!(config.api_key.is_none());
    }
    
    #[test]
    fn test_claude_api_key_validation() {
        // Invalid key
        let result = ClaudeProvider::new("invalid-key".to_string(), None);
        assert!(result.is_err());
        
        // Valid key format
        let result = ClaudeProvider::new("sk-ant-api03-test".to_string(), None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_openai_api_key_validation() {
        // Invalid key
        let result = OpenAiProvider::new("invalid-key".to_string(), None);
        assert!(result.is_err());
        
        // Valid key format
        let result = OpenAiProvider::new("sk-test12345".to_string(), None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_provider_manager() {
        let manager = ProviderManager::new();
        assert_eq!(manager.current_provider(), AiProviderType::Local);
        assert!(!manager.is_ready()); // Local model not loaded
    }
}
