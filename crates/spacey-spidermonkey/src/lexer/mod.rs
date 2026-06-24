//! Lexical analysis (tokenization) for JavaScript source code.
//!
//! The lexer transforms JavaScript source text into a stream of tokens
//! that can be consumed by the parser.

mod scanner;
mod token;

pub use scanner::Scanner;
pub use token::{Span, Token, TokenKind};
