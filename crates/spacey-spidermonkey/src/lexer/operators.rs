//! Operator scanning documentation.
//!
//! This module documents the operator scanning logic in `scanner.rs`.
//! The lexer handles multi-character operators by looking ahead.
//!
//! ## Operator Categories
//!
//! ### Arithmetic Operators
//!
//! | Token | Method | Variants |
//! |-------|--------|----------|
//! | `+` | `scan_plus` | `+`, `++`, `+=` |
//! | `-` | `scan_minus` | `-`, `--`, `-=` |
//! | `*` | `scan_star` | `*`, `**`, `*=`, `**=` |
//! | `/` | `scan_slash` | `/`, `/=`, `//...`, `/*...*/` |
//! | `%` | `scan_percent` | `%`, `%=` |
//!
//! ### Comparison Operators
//!
//! | Token | Method | Variants |
//! |-------|--------|----------|
//! | `<` | `scan_less_than` | `<`, `<=`, `<<`, `<<=` |
//! | `>` | `scan_greater_than` | `>`, `>=`, `>>`, `>>>`, `>>=`, `>>>=` |
//! | `=` | `scan_equal` | `=`, `==`, `===`, `=>` |
//! | `!` | `scan_bang` | `!`, `!=`, `!==` |
//!
//! ### Bitwise Operators
//!
//! | Token | Method | Variants |
//! |-------|--------|----------|
//! | `&` | `scan_ampersand` | `&`, `&&`, `&=`, `&&=` |
//! | `\|` | `scan_pipe` | `\|`, `\|\|`, `\|=`, `\|\|=` |
//! | `^` | `scan_caret` | `^`, `^=` |
//!
//! ### Other Operators
//!
//! | Token | Method | Variants |
//! |-------|--------|----------|
//! | `?` | `scan_question` | `?`, `?.`, `??`, `??=` |
//! | `.` | `scan_dot` | `.`, `...`, `.123` (number) |
//!
//! ## Lookahead Logic
//!
//! Multi-character operators use peek to determine the full token:
//!
//! ```text
//! // For input "==="
//! scan_equal():
//!   consume '='
//!   peek() returns '='  -> not just '='
//!   advance()
//!   peek() returns '='  -> it's '==='
//!   advance()
//!   return StrictEqual
//! ```
//!
//! ## Comment Handling
//!
//! The `/` character can start:
//! - Division: `a / b`
//! - Division assignment: `a /= b`
//! - Single-line comment: `// comment`
//! - Multi-line comment: `/* comment */`
//! - Regular expression: `/pattern/flags`
//!
//! The scanner handles `//` and `/*` in `skip_whitespace_and_comments`.
//! Regex literals require context from the parser (`scan_regex_literal`).

// This module serves as documentation. The actual implementation is in scanner.rs.

#[cfg(test)]
mod tests {
    use crate::lexer::{Scanner, TokenKind};

    fn scan_single(src: &str) -> TokenKind {
        let mut scanner = Scanner::new(src);
        scanner.next_token().kind
    }

    #[test]
    fn test_plus_operators() {
        assert!(matches!(scan_single("+"), TokenKind::Plus));
        assert!(matches!(scan_single("++"), TokenKind::PlusPlus));
        assert!(matches!(scan_single("+="), TokenKind::PlusEqual));
    }

    #[test]
    fn test_minus_operators() {
        assert!(matches!(scan_single("-"), TokenKind::Minus));
        assert!(matches!(scan_single("--"), TokenKind::MinusMinus));
        assert!(matches!(scan_single("-="), TokenKind::MinusEqual));
    }

    #[test]
    fn test_star_operators() {
        assert!(matches!(scan_single("*"), TokenKind::Star));
        assert!(matches!(scan_single("**"), TokenKind::StarStar));
        assert!(matches!(scan_single("*="), TokenKind::StarEqual));
    }

    #[test]
    fn test_slash_operators() {
        assert!(matches!(scan_single("/"), TokenKind::Slash));
        assert!(matches!(scan_single("/="), TokenKind::SlashEqual));
    }

    #[test]
    fn test_less_than_operators() {
        assert!(matches!(scan_single("<"), TokenKind::LessThan));
        assert!(matches!(scan_single("<="), TokenKind::LessThanEqual));
        assert!(matches!(scan_single("<<"), TokenKind::LeftShift));
        assert!(matches!(scan_single("<<="), TokenKind::LeftShiftEqual));
    }

    #[test]
    fn test_greater_than_operators() {
        assert!(matches!(scan_single(">"), TokenKind::GreaterThan));
        assert!(matches!(scan_single(">="), TokenKind::GreaterThanEqual));
        assert!(matches!(scan_single(">>"), TokenKind::RightShift));
        assert!(matches!(scan_single(">>>"), TokenKind::UnsignedRightShift));
    }

    #[test]
    fn test_equal_operators() {
        assert!(matches!(scan_single("="), TokenKind::Equal));
        assert!(matches!(scan_single("=="), TokenKind::EqualEqual));
        assert!(matches!(scan_single("==="), TokenKind::StrictEqual));
        assert!(matches!(scan_single("=>"), TokenKind::Arrow));
    }

    #[test]
    fn test_bang_operators() {
        assert!(matches!(scan_single("!"), TokenKind::Bang));
        assert!(matches!(scan_single("!="), TokenKind::NotEqual));
        assert!(matches!(scan_single("!=="), TokenKind::StrictNotEqual));
    }

    #[test]
    fn test_ampersand_operators() {
        assert!(matches!(scan_single("&"), TokenKind::Ampersand));
        assert!(matches!(scan_single("&&"), TokenKind::AmpersandAmpersand));
        assert!(matches!(scan_single("&="), TokenKind::AmpersandEqual));
    }

    #[test]
    fn test_pipe_operators() {
        assert!(matches!(scan_single("|"), TokenKind::Pipe));
        assert!(matches!(scan_single("||"), TokenKind::PipePipe));
        assert!(matches!(scan_single("|="), TokenKind::PipeEqual));
    }

    #[test]
    fn test_caret_operators() {
        assert!(matches!(scan_single("^"), TokenKind::Caret));
        assert!(matches!(scan_single("^="), TokenKind::CaretEqual));
    }

    #[test]
    fn test_question_operators() {
        assert!(matches!(scan_single("?"), TokenKind::Question));
        assert!(matches!(scan_single("?."), TokenKind::QuestionDot));
        assert!(matches!(scan_single("??"), TokenKind::QuestionQuestion));
    }

    #[test]
    fn test_dot_operators() {
        assert!(matches!(scan_single("."), TokenKind::Dot));
        assert!(matches!(scan_single("..."), TokenKind::Ellipsis));
    }

    #[test]
    fn test_percent_operators() {
        assert!(matches!(scan_single("%"), TokenKind::Percent));
        assert!(matches!(scan_single("%="), TokenKind::PercentEqual));
    }
}
