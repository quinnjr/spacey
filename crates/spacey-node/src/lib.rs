// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! # spacey-node
//!
//! A Node.js-compatible runtime built on the Spacey JavaScript engine.
//!
//! This crate provides a complete server-side JavaScript runtime with Node.js API
//! compatibility, including:
//!
//! - CommonJS module system (`require()`)
//! - ESM module system (`import`/`export`)
//! - **Native TypeScript execution** (no transpilation step)
//! - Node.js globals (`process`, `Buffer`, `__dirname`, `__filename`)
//! - Built-in modules (`fs`, `path`, `http`, `crypto`, etc.)
//! - Event loop with async I/O
//!
//! ## TypeScript Support
//!
//! Spacey natively parses and executes TypeScript by stripping type annotations
//! at parse time. No separate transpilation step is required:
//!
//! ```rust,ignore
//! use spacey_node::NodeRuntime;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut runtime = NodeRuntime::new(vec![]);
//!     // TypeScript files are automatically detected and handled
//!     runtime.run_file(Path::new("server.ts")).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use spacey_node::NodeRuntime;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut runtime = NodeRuntime::new(vec![]);
//!     runtime.run_file(Path::new("server.js")).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! # Run a JavaScript file
//! spacey-node server.js
//!
//! # Run a TypeScript file (native execution)
//! spacey-node server.ts
//!
//! # Start REPL
//! spacey-node --repl
//!
//! # Evaluate inline script
//! spacey-node -e "console.log('Hello, World!')"
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod globals;
pub mod module_system;
pub mod modules;
pub mod runtime;
pub mod typescript;

// Re-exports
pub use error::{NodeError, Result};
pub use runtime::NodeRuntime;
pub use typescript::{ALL_EXTENSIONS, TS_EXTENSIONS, is_jsx_file, is_typescript_file};

/// Version of the spacey-node runtime
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Node.js API version compatibility target
pub const NODE_API_VERSION: &str = "20.0.0";
