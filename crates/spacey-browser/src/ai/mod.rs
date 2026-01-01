//! AI Module - Embedded AI agent for browser automation
//!
//! This module provides an AI-powered browser assistant using the Phi-3 model.
//! It supports agentic browsing through a ReAct (Reasoning + Acting) loop.
//!
//! # BYOK (Bring Your Own Key) Support
//!
//! Users can choose between:
//! - **Local**: Phi-3 model running entirely on-device (free, private)
//! - **Claude**: Anthropic's API (requires API key)
//! - **OpenAI**: ChatGPT API (requires API key)

pub mod agent;
pub mod memory;
pub mod model;
pub mod planner;
pub mod provider;
pub mod tools;

// Re-exports for convenience
pub use agent::{AgentConfig, AiAgent, ActionRecord, AgentState};
pub use memory::PageContext;
pub use provider::{
    AiProvider, AiProviderConfig, AiProviderType, ChatMessage, ClaudeProvider,
    LocalProvider, MessageRole, OpenAiProvider, ProviderError, ProviderManager,
};
pub use tools::{BrowserTool, ToolResult};
