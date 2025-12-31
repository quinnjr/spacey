//! AI Module - Embedded AI agent for browser automation
//!
//! This module provides an AI-powered browser assistant using the Phi-3 model.
//! It supports agentic browsing through a ReAct (Reasoning + Acting) loop.

pub mod agent;
pub mod memory;
pub mod model;
pub mod planner;
pub mod tools;

// Re-exports for convenience
pub use agent::{AgentConfig, AiAgent, ActionRecord, AgentState};
pub use memory::PageContext;
pub use tools::{BrowserTool, ToolResult};
