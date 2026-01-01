//! Spacey Browser Library
//!
//! This crate provides the browser functionality including:
//! - AI-powered browsing assistant (Phi-3)
//! - HTML parsing and rendering
//! - JavaScript execution via Spacey engine
//! - Firefox-compatible extension system with FULL webRequest support
//! - Built-in Spacey Shield privacy protection
//! - Steam integration for licensing and distribution

pub mod ai;
pub mod ai_ui;
pub mod browser;
pub mod extensions;
pub mod extensions_ui;
pub mod licensing;
pub mod page;
pub mod renderer;
pub mod shield;

pub use browser::Browser;
pub use ai::{AiAgent, AgentConfig};
pub use extensions::{ExtensionManager, Extension, ExtensionManifest};
pub use licensing::{LicenseManager, LicenseStatus, LicenseUser, SpaceyEntitlement};
pub use shield::{SpaceyShield, ShieldLevel, ShieldStats};
