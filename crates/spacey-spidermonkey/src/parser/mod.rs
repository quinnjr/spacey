#![allow(clippy::module_inception)]
//! Parser for JavaScript source code.
//!
//! Transforms a stream of tokens into an Abstract Syntax Tree (AST).
//!
//! ## Structure
//!
//! - `parser` - Main recursive descent parser implementation
//!
//! ## Documentation Submodules
//!
//! The following submodules provide documentation and additional tests
//! for specific parsing areas:
//!
//! - `statements` - Statement parsing (if, for, while, etc.)
//! - `expressions` - Expression parsing (operators, literals, calls)
//! - `typescript` - TypeScript extensions (type annotations, generics)
//!
//! ## Usage
//!
//! ```rust
//! use spacey_spidermonkey::parser::Parser;
//!
//! let mut parser = Parser::new("let x = 1 + 2;");
//! let program = parser.parse_program().expect("Should parse");
//! ```
//!
//! For TypeScript:
//!
//! ```rust
//! use spacey_spidermonkey::parser::Parser;
//!
//! let mut parser = Parser::new_typescript("let x: number = 42;");
//! let program = parser.parse_program().expect("Should parse");
//! ```

mod parser;

// Documentation and test submodules
pub mod expressions;
pub mod statements;
pub mod typescript;

pub use parser::Parser;
