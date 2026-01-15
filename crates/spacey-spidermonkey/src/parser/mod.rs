//! Parser for JavaScript source code.
//!
//! Transforms a stream of tokens into an Abstract Syntax Tree (AST).

#[allow(clippy::module_inception)]
mod parser;

pub use parser::Parser;
