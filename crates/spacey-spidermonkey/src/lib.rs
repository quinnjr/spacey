// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! # spacey-spidermonkey
//!
//! A JavaScript engine inspired by Mozilla's SpiderMonkey, implemented in Rust.
//!
//! ## Overview
//!
//! This crate provides a complete JavaScript execution environment including:
//! - Lexer and parser for ECMAScript 2024+
//! - Bytecode compiler and interpreter
//! - Garbage-collected runtime with parallel GC
//! - Built-in objects and standard library
//! - Async/parallel execution support
//!
//! ## Quick Start (Sync)
//!
//! ```rust,ignore
//! use spacey_spidermonkey::{Engine, Value};
//!
//! let mut engine = Engine::new();
//! let result = engine.eval("1 + 2")?;
//! assert_eq!(result, Value::Number(3.0));
//! ```
//!
//! ## Async Example
//!
//! ```rust,ignore
//! use spacey_spidermonkey::AsyncEngine;
//!
//! #[tokio::main]
//! async fn main() {
//!     let engine = AsyncEngine::new();
//!     let result = engine.eval_file("script.js").await.unwrap();
//!     println!("{}", result);
//! }
//! ```
//!
//! ## Features
//!
//! - `async` - Enables async engine APIs using tokio (default)
//! - `parallel` - Enables parallel compilation and GC using rayon (default)

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules
pub mod ast;
pub mod builtins;
pub mod compiler;
pub mod gc;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod vm;

// Async and parallel modules
#[cfg(feature = "async")]
pub mod async_engine;

// Re-exports for convenience
pub use runtime::context::Context;
pub use runtime::value::Value;

#[cfg(feature = "async")]
pub use async_engine::AsyncEngine;

#[cfg(all(feature = "async", feature = "parallel"))]
pub use async_engine::ParallelExecutor;

use compiler::Compiler;
use parser::Parser;
use vm::VM;

/// The main JavaScript engine instance.
///
/// Encapsulates the entire JavaScript execution environment including
/// the heap, global object, and execution state.
pub struct Engine {
    #[allow(dead_code)]
    context: Context,
    vm: VM,
}

impl Engine {
    /// Creates a new JavaScript engine instance with default configuration.
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            vm: VM::new(),
        }
    }

    /// Evaluates JavaScript source code and returns the result.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript source code to evaluate
    ///
    /// # Returns
    ///
    /// The result of evaluating the expression, or an error if parsing
    /// or execution fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new();
    /// let result = engine.eval("2 + 2")?;
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        // 1. Parse source into AST (lexer is used internally by parser)
        let mut parser = Parser::new(source);
        let ast = parser.parse_program()?;

        // 2. Compile AST to bytecode
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast)?;

        // 3. Execute bytecode in VM
        self.vm.execute(&bytecode)
    }

    /// Evaluates JavaScript source code from a file.
    pub fn eval_file(&mut self, path: &std::path::Path) -> Result<Value, Error> {
        let source = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;
        self.eval(&source)
    }

    /// Evaluates TypeScript source code and returns the result.
    ///
    /// TypeScript syntax (type annotations, interfaces, type aliases, etc.)
    /// is parsed and stripped at parse time, producing a JavaScript AST
    /// that is then compiled and executed.
    ///
    /// # Arguments
    ///
    /// * `source` - The TypeScript source code to evaluate
    ///
    /// # Returns
    ///
    /// The result of evaluating the expression, or an error if parsing
    /// or execution fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new();
    /// let result = engine.eval_typescript("const x: number = 2 + 2; x;")?;
    /// assert_eq!(result, Value::Number(4.0));
    /// ```
    pub fn eval_typescript(&mut self, source: &str) -> Result<Value, Error> {
        // 1. Parse source into AST with TypeScript mode enabled
        let mut parser = Parser::new(source);
        parser.set_typescript_mode(true);
        let ast = parser.parse_program()?;

        // 2. Compile AST to bytecode
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast)?;

        // 3. Execute bytecode in VM
        self.vm.execute(&bytecode)
    }

    /// Evaluates TypeScript source code from a file.
    ///
    /// The file is treated as TypeScript regardless of extension.
    /// For automatic detection based on extension, use `eval_file_auto()`.
    pub fn eval_file_typescript(&mut self, path: &std::path::Path) -> Result<Value, Error> {
        let source = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;
        self.eval_typescript(&source)
    }

    /// Evaluates source code from a file, automatically detecting
    /// whether it's TypeScript or JavaScript based on the file extension.
    ///
    /// TypeScript extensions: `.ts`, `.tsx`, `.mts`, `.cts`
    /// JavaScript extensions: `.js`, `.jsx`, `.mjs`, `.cjs` (and others)
    pub fn eval_file_auto(&mut self, path: &std::path::Path) -> Result<Value, Error> {
        let source = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;

        // Detect TypeScript by extension
        let is_typescript = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("ts" | "tsx" | "mts" | "cts")
        );

        if is_typescript {
            self.eval_typescript(&source)
        } else {
            self.eval(&source)
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during JavaScript execution.
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax error during parsing
    SyntaxError(String),
    /// Type error during execution
    TypeError(String),
    /// Reference error (undefined variable)
    ReferenceError(String),
    /// Range error (out of bounds, etc.)
    RangeError(String),
    /// Internal engine error
    InternalError(String),
    /// I/O error
    Io(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            Error::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Error::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            Error::RangeError(msg) => write!(f, "RangeError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Error::Io(msg) => write!(f, "IOError: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(matches!(engine.context, _));
    }

    #[test]
    fn test_eval_number_literal() {
        let mut engine = Engine::new();
        let result = engine.eval("42;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut engine = Engine::new();
        let result = engine.eval("1 + 2 * 3;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0));
    }

    #[test]
    fn test_eval_comparison() {
        let mut engine = Engine::new();
        let result = engine.eval("5 > 3;").unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_eval_string_literal() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\";").unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_eval_boolean_literal() {
        let mut engine = Engine::new();
        let result = engine.eval("true;").unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_eval_with_comments() {
        let mut engine = Engine::new();
        // Single line comment
        let result = engine.eval("1 + 2; // this is ignored").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Multi-line comment
        let result = engine.eval("/* add numbers */ 5 * 3;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 15.0));
    }

    #[test]
    fn test_eval_variable_declaration() {
        let mut engine = Engine::new();
        // Simple variable declaration and use
        let result = engine.eval("let x = 10; x;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));

        // Multiple declarations
        let result = engine.eval("let a = 5; let b = 3; a + b;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));

        // Variable assignment
        let result = engine.eval("let x = 1; x = 2; x;").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_builtin_print() {
        let mut engine = Engine::new();
        // print() should be available and return undefined
        let result = engine.eval("print(\"hello\");").unwrap();
        assert!(matches!(result, Value::Undefined));
    }
}
