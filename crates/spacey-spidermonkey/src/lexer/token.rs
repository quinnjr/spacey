//! Token definitions for the JavaScript lexer.

/// A span in the source code, representing a range of characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the length of this span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if this span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The span in the source code
    pub span: Span,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The different kinds of tokens in JavaScript.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum TokenKind {
    // Literals
    /// Numeric literal (integer or floating point)
    Number(f64),
    /// BigInt literal
    BigInt(String),
    /// String literal
    String(String),
    /// Template literal part
    Template(String),
    /// Regular expression literal
    RegExp {
        pattern: String,
        flags: String,
    },
    /// Boolean true
    True,
    /// Boolean false
    False,
    /// null
    Null,

    // Identifiers and Keywords
    /// Identifier
    Identifier(String),
    /// Private identifier (#name)
    PrivateIdentifier(String),

    // Keywords
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    Let,
    New,
    Return,
    Static,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
    Async,

    // TypeScript Keywords
    /// type (TypeScript)
    Type,
    /// interface (TypeScript)
    Interface,
    /// namespace (TypeScript)
    Namespace,
    /// declare (TypeScript)
    Declare,
    /// readonly (TypeScript)
    Readonly,
    /// abstract (TypeScript)
    Abstract,
    /// implements (TypeScript)
    Implements,
    /// private (TypeScript - also valid JS class field)
    Private,
    /// protected (TypeScript)
    Protected,
    /// public (TypeScript)
    Public,
    /// as (TypeScript type assertion)
    As,
    /// is (TypeScript type predicate)
    Is,
    /// keyof (TypeScript)
    Keyof,
    /// infer (TypeScript)
    Infer,
    /// never (TypeScript)
    Never,
    /// unknown (TypeScript)
    Unknown,
    /// any (TypeScript)
    Any,
    /// asserts (TypeScript)
    Asserts,
    /// override (TypeScript)
    Override,
    /// satisfies (TypeScript)
    Satisfies,
    /// out (TypeScript variance modifier)
    Out,
    /// accessor (TypeScript)
    Accessor,

    // Punctuation
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// .
    Dot,
    /// ...
    Ellipsis,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// <
    LessThan,
    /// >
    GreaterThan,
    /// <=
    LessThanEqual,
    /// >=
    GreaterThanEqual,
    /// ==
    EqualEqual,
    /// !=
    NotEqual,
    /// ===
    StrictEqual,
    /// !==
    StrictNotEqual,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// **
    StarStar,
    /// ++
    PlusPlus,
    /// --
    MinusMinus,
    /// <<
    LeftShift,
    /// >>
    RightShift,
    /// >>>
    UnsignedRightShift,
    /// &
    Ampersand,
    /// |
    Pipe,
    /// ^
    Caret,
    /// !
    Bang,
    /// ~
    Tilde,
    /// &&
    AmpersandAmpersand,
    /// ||
    PipePipe,
    /// ??
    QuestionQuestion,
    /// ?
    Question,
    /// ?.
    QuestionDot,
    /// :
    Colon,
    /// =
    Equal,
    /// +=
    PlusEqual,
    /// -=
    MinusEqual,
    /// *=
    StarEqual,
    /// /=
    SlashEqual,
    /// %=
    PercentEqual,
    /// **=
    StarStarEqual,
    /// <<=
    LeftShiftEqual,
    /// >>=
    RightShiftEqual,
    /// >>>=
    UnsignedRightShiftEqual,
    /// &=
    AmpersandEqual,
    /// |=
    PipeEqual,
    /// ^=
    CaretEqual,
    /// &&=
    AmpersandAmpersandEqual,
    /// ||=
    PipePipeEqual,
    /// ??=
    QuestionQuestionEqual,
    /// =>
    Arrow,
    /// @ (decorator)
    At,

    // Special
    /// End of file
    Eof,
    /// Invalid token (for error recovery)
    Invalid,
}

impl TokenKind {
    /// Returns true if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Await
                | TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Class
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Default
                | TokenKind::Delete
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::Extends
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::Function
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Instanceof
                | TokenKind::Let
                | TokenKind::New
                | TokenKind::Return
                | TokenKind::Static
                | TokenKind::Super
                | TokenKind::Switch
                | TokenKind::This
                | TokenKind::Throw
                | TokenKind::Try
                | TokenKind::Typeof
                | TokenKind::Var
                | TokenKind::Void
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
                | TokenKind::Async
        )
    }

    /// Returns true if this token is a TypeScript keyword.
    pub fn is_typescript_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Type
                | TokenKind::Interface
                | TokenKind::Namespace
                | TokenKind::Declare
                | TokenKind::Readonly
                | TokenKind::Abstract
                | TokenKind::Implements
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Public
                | TokenKind::As
                | TokenKind::Is
                | TokenKind::Keyof
                | TokenKind::Infer
                | TokenKind::Never
                | TokenKind::Unknown
                | TokenKind::Any
                | TokenKind::Asserts
                | TokenKind::Override
                | TokenKind::Satisfies
                | TokenKind::Out
                | TokenKind::Accessor
        )
    }

    /// Returns true if this token is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::Number(_)
                | TokenKind::BigInt(_)
                | TokenKind::String(_)
                | TokenKind::Template(_)
                | TokenKind::RegExp { .. }
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 10);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(5, 15);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(5, 5);
        let non_empty = Span::new(5, 10);

        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(0, 10);
        let span2 = Span::new(0, 10);
        let span3 = Span::new(0, 5);

        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_span_clone() {
        let span = Span::new(0, 10);
        let cloned = span;
        assert_eq!(span, cloned);
    }

    #[test]
    fn test_token_new() {
        let token = Token::new(TokenKind::Number(42.0), Span::new(0, 2));
        assert_eq!(token.kind, TokenKind::Number(42.0));
        assert_eq!(token.span, Span::new(0, 2));
    }

    #[test]
    fn test_token_equality() {
        let t1 = Token::new(TokenKind::Plus, Span::new(0, 1));
        let t2 = Token::new(TokenKind::Plus, Span::new(0, 1));
        let t3 = Token::new(TokenKind::Minus, Span::new(0, 1));

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_token_clone() {
        let token = Token::new(TokenKind::Identifier("x".to_string()), Span::new(0, 1));
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[test]
    fn test_is_keyword_true() {
        assert!(TokenKind::If.is_keyword());
        assert!(TokenKind::Else.is_keyword());
        assert!(TokenKind::For.is_keyword());
        assert!(TokenKind::While.is_keyword());
        assert!(TokenKind::Function.is_keyword());
        assert!(TokenKind::Const.is_keyword());
        assert!(TokenKind::Let.is_keyword());
        assert!(TokenKind::Var.is_keyword());
        assert!(TokenKind::Return.is_keyword());
        assert!(TokenKind::Break.is_keyword());
        assert!(TokenKind::Continue.is_keyword());
        assert!(TokenKind::Class.is_keyword());
        assert!(TokenKind::Extends.is_keyword());
        assert!(TokenKind::Super.is_keyword());
        assert!(TokenKind::This.is_keyword());
        assert!(TokenKind::New.is_keyword());
        assert!(TokenKind::Try.is_keyword());
        assert!(TokenKind::Catch.is_keyword());
        assert!(TokenKind::Finally.is_keyword());
        assert!(TokenKind::Throw.is_keyword());
        assert!(TokenKind::Typeof.is_keyword());
        assert!(TokenKind::Instanceof.is_keyword());
        assert!(TokenKind::In.is_keyword());
        assert!(TokenKind::Delete.is_keyword());
        assert!(TokenKind::Void.is_keyword());
        assert!(TokenKind::Await.is_keyword());
        assert!(TokenKind::Async.is_keyword());
        assert!(TokenKind::Yield.is_keyword());
        assert!(TokenKind::Switch.is_keyword());
        assert!(TokenKind::Case.is_keyword());
        assert!(TokenKind::Default.is_keyword());
        assert!(TokenKind::Do.is_keyword());
        assert!(TokenKind::With.is_keyword());
        assert!(TokenKind::Debugger.is_keyword());
        assert!(TokenKind::Enum.is_keyword());
        assert!(TokenKind::Export.is_keyword());
        assert!(TokenKind::Import.is_keyword());
        assert!(TokenKind::Static.is_keyword());
    }

    #[test]
    fn test_is_keyword_false() {
        assert!(!TokenKind::Plus.is_keyword());
        assert!(!TokenKind::Number(42.0).is_keyword());
        assert!(!TokenKind::String("hello".to_string()).is_keyword());
        assert!(!TokenKind::Identifier("x".to_string()).is_keyword());
        assert!(!TokenKind::True.is_keyword());
        assert!(!TokenKind::False.is_keyword());
        assert!(!TokenKind::Null.is_keyword());
        assert!(!TokenKind::Eof.is_keyword());
    }

    #[test]
    fn test_is_literal_true() {
        assert!(TokenKind::Number(42.0).is_literal());
        assert!(TokenKind::BigInt("123".to_string()).is_literal());
        assert!(TokenKind::String("hello".to_string()).is_literal());
        assert!(TokenKind::Template("template".to_string()).is_literal());
        assert!(
            TokenKind::RegExp {
                pattern: ".*".to_string(),
                flags: "g".to_string()
            }
            .is_literal()
        );
        assert!(TokenKind::True.is_literal());
        assert!(TokenKind::False.is_literal());
        assert!(TokenKind::Null.is_literal());
    }

    #[test]
    fn test_is_literal_false() {
        assert!(!TokenKind::Plus.is_literal());
        assert!(!TokenKind::If.is_literal());
        assert!(!TokenKind::Identifier("x".to_string()).is_literal());
        assert!(!TokenKind::LeftBrace.is_literal());
        assert!(!TokenKind::Eof.is_literal());
    }

    #[test]
    fn test_token_kind_debug() {
        let kind = TokenKind::Number(42.0);
        let debug = format!("{:?}", kind);
        assert!(debug.contains("Number"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_token_kind_clone() {
        let kind = TokenKind::String("hello".to_string());
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_all_punctuation_tokens() {
        // Test that punctuation tokens exist and can be compared
        let tokens = vec![
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::Dot,
            TokenKind::Ellipsis,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::LessThan,
            TokenKind::GreaterThan,
            TokenKind::LessThanEqual,
            TokenKind::GreaterThanEqual,
            TokenKind::EqualEqual,
            TokenKind::NotEqual,
            TokenKind::StrictEqual,
            TokenKind::StrictNotEqual,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::StarStar,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
            TokenKind::LeftShift,
            TokenKind::RightShift,
            TokenKind::UnsignedRightShift,
            TokenKind::Ampersand,
            TokenKind::Pipe,
            TokenKind::Caret,
            TokenKind::Bang,
            TokenKind::Tilde,
            TokenKind::AmpersandAmpersand,
            TokenKind::PipePipe,
            TokenKind::QuestionQuestion,
            TokenKind::Question,
            TokenKind::QuestionDot,
            TokenKind::Colon,
            TokenKind::Equal,
            TokenKind::PlusEqual,
            TokenKind::MinusEqual,
            TokenKind::StarEqual,
            TokenKind::SlashEqual,
            TokenKind::PercentEqual,
            TokenKind::StarStarEqual,
            TokenKind::LeftShiftEqual,
            TokenKind::RightShiftEqual,
            TokenKind::UnsignedRightShiftEqual,
            TokenKind::AmpersandEqual,
            TokenKind::PipeEqual,
            TokenKind::CaretEqual,
            TokenKind::AmpersandAmpersandEqual,
            TokenKind::PipePipeEqual,
            TokenKind::QuestionQuestionEqual,
            TokenKind::Arrow,
        ];

        // All punctuation tokens should not be keywords or literals
        for token in tokens {
            assert!(!token.is_keyword());
            assert!(!token.is_literal());
        }
    }

    #[test]
    fn test_special_tokens() {
        assert!(!TokenKind::Eof.is_keyword());
        assert!(!TokenKind::Eof.is_literal());
        assert!(!TokenKind::Invalid.is_keyword());
        assert!(!TokenKind::Invalid.is_literal());
    }

    #[test]
    fn test_private_identifier() {
        let private = TokenKind::PrivateIdentifier("name".to_string());
        assert!(!private.is_keyword());
        assert!(!private.is_literal());
    }
}
