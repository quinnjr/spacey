// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Spacey - A JavaScript engine inspired by SpiderMonkey, written in Rust
//!
//! This crate provides a JavaScript execution engine with a complete pipeline
//! from source parsing to bytecode execution. It is designed to be compatible
//! with modern ECMAScript standards (ES2022+).
//!
//! # Quick Start
//!
//! ```
//! use spacey::Engine;
//!
//! let mut engine = Engine::new();
//! let result = engine.eval("2 + 3 * 4").unwrap();
//! // result == Value::Number(14.0)
//! ```
//!
//! # Architecture
//!
//! The engine follows a staged pipeline:
//!
//! ```text
//! Source Code → Lexer → Parser → AST → Compiler → Bytecode → VM → Result
//! ```

// Re-export the core engine library
pub use spacey_spidermonkey::*;
