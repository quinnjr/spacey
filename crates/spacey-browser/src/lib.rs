//! Spacey Browser Library
//!
//! This crate provides the browser functionality including:
//! - AI-powered browsing assistant (Phi-3)
//! - HTML parsing and rendering
//! - JavaScript execution via Spacey engine

pub mod ai;
pub mod ai_ui;
pub mod browser;
pub mod page;
pub mod renderer;

pub use browser::Browser;
pub use ai::{AiAgent, AgentConfig};
