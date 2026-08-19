//! The scanner that produces tokens from source text.

use super::{Span, Token, TokenKind};

/// A scanner that tokenizes JavaScript source code.
pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    /// Enable TypeScript syntax support
    typescript_mode: bool,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            typescript_mode: false,
        }
    }

    /// Creates a new scanner with TypeScript mode enabled.
    pub fn new_typescript(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            typescript_mode: true,
        }
    }

    /// Enable or disable TypeScript mode.
    pub fn set_typescript_mode(&mut self, enabled: bool) {
        self.typescript_mode = enabled;
    }

    /// Returns whether TypeScript mode is enabled.
    pub fn is_typescript_mode(&self) -> bool {
        self.typescript_mode
    }

    /// Peeks at the next token without consuming it.
    /// Note: This is relatively expensive as it clones internal state.
    pub fn peek_token(&self) -> Token {
        // Create a copy of the scanner state
        let mut scanner_copy = if self.typescript_mode {
            Scanner::new_typescript(self.source)
        } else {
            Scanner::new(self.source)
        };
        scanner_copy.current_pos = self.current_pos;
        scanner_copy.chars = self.source[self.current_pos..].char_indices().peekable();
        // Adjust positions in the cloned iterator
        scanner_copy.next_token()
    }

    /// Returns the next token from the source.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.current_pos;

        let Some((_pos, ch)) = self.advance() else {
            return Token::new(TokenKind::Eof, Span::new(start, start));
        };

        let kind = match ch {
            // Single-character tokens
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '~' => TokenKind::Tilde,

            // Multi-character tokens
            '.' => self.scan_dot(),
            '+' => self.scan_plus(),
            '-' => self.scan_minus(),
            '*' => self.scan_star(),
            '/' => self.scan_slash(),
            '%' => self.scan_percent(),
            '<' => self.scan_less_than(),
            '>' => self.scan_greater_than(),
            '=' => self.scan_equal(),
            '!' => self.scan_bang(),
            '&' => self.scan_ampersand(),
            '|' => self.scan_pipe(),
            '^' => self.scan_caret(),
            '?' => self.scan_question(),

            // String literals
            '"' | '\'' => self.scan_string(ch),

            // Template literals
            '`' => self.scan_template(),

            // Numbers
            '0'..='9' => self.scan_number(ch),

            // Identifiers and keywords
            _ if is_id_start(ch) => self.scan_identifier(ch),

            // Private identifiers
            '#' => self.scan_private_identifier(),

            // Decorators (TypeScript)
            '@' => TokenKind::At,

            _ => TokenKind::Invalid,
        };

        Token::new(kind, Span::new(start, self.current_pos))
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.current_pos = pos + ch.len_utf8();
        }
        result
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().map(|(_, ch)| ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\n' | '\r') => {
                    self.advance();
                }
                Some('/') => {
                    match self.peek_next() {
                        Some('/') => {
                            // Single-line comment: skip until end of line
                            self.advance(); // consume first '/'
                            self.advance(); // consume second '/'
                            while let Some(ch) = self.peek() {
                                if ch == '\n' || ch == '\r' {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        Some('*') => {
                            // Multi-line comment: skip until */
                            self.advance(); // consume '/'
                            self.advance(); // consume '*'
                            let mut prev = ' ';
                            while let Some(ch) = self.peek() {
                                self.advance();
                                if prev == '*' && ch == '/' {
                                    break;
                                }
                                prev = ch;
                            }
                        }
                        _ => break, // Not a comment, it's a division operator
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_dot(&mut self) -> TokenKind {
        if self.peek() == Some('.') {
            self.advance();
            if self.peek() == Some('.') {
                self.advance();
                TokenKind::Ellipsis
            } else {
                // Invalid: ".." is not valid
                TokenKind::Invalid
            }
        } else {
            TokenKind::Dot
        }
    }

    fn scan_plus(&mut self) -> TokenKind {
        match self.peek() {
            Some('+') => {
                self.advance();
                TokenKind::PlusPlus
            }
            Some('=') => {
                self.advance();
                TokenKind::PlusEqual
            }
            _ => TokenKind::Plus,
        }
    }

    fn scan_minus(&mut self) -> TokenKind {
        match self.peek() {
            Some('-') => {
                self.advance();
                TokenKind::MinusMinus
            }
            Some('=') => {
                self.advance();
                TokenKind::MinusEqual
            }
            _ => TokenKind::Minus,
        }
    }

    fn scan_star(&mut self) -> TokenKind {
        match self.peek() {
            Some('*') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StarStarEqual
                } else {
                    TokenKind::StarStar
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::StarEqual
            }
            _ => TokenKind::Star,
        }
    }

    fn scan_slash(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                TokenKind::SlashEqual
            }
            _ => TokenKind::Slash,
        }
    }

    /// Scan a regex literal starting after the opening '/'.
    /// This should be called when the parser determines a '/' starts a regex.
    /// The scanner should be positioned right after the '/'.
    pub fn scan_regex_literal(&mut self) -> Token {
        let start = self.current_pos - 1; // Include the '/' we already consumed
        let mut pattern = String::new();
        let mut in_char_class = false;

        // Scan the pattern until we find the closing '/'
        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => {
                    // Unterminated regex literal
                    return Token::new(TokenKind::Invalid, Span::new(start, self.current_pos));
                }
                Some('/') if !in_char_class => {
                    // End of pattern
                    self.advance();
                    break;
                }
                Some('\\') => {
                    // Escape sequence - include both characters
                    self.advance();
                    pattern.push('\\');
                    if let Some((_, ch)) = self.advance() {
                        pattern.push(ch);
                    }
                }
                Some('[') => {
                    in_char_class = true;
                    if let Some((_, ch)) = self.advance() {
                        pattern.push(ch);
                    }
                }
                Some(']') => {
                    in_char_class = false;
                    if let Some((_, ch)) = self.advance() {
                        pattern.push(ch);
                    }
                }
                Some(ch) => {
                    pattern.push(ch);
                    self.advance();
                }
            }
        }

        // Scan the flags
        let mut flags = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                flags.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::RegExp { pattern, flags },
            Span::new(start, self.current_pos),
        )
    }

    /// Check if we can rescan the last token as a regex literal.
    /// This is used by the parser when it realizes a '/' should be a regex start.
    pub fn rescan_as_regex(&mut self, slash_start: usize) -> Token {
        // Position ourselves at the character after the '/'
        // The slash_start is the position of the '/'
        // We need to reset to just after the slash

        // Create a new iterator from the slash position + 1
        self.chars = self.source[slash_start + 1..].char_indices().peekable();
        self.current_pos = slash_start + 1;

        self.scan_regex_literal()
    }

    fn scan_percent(&mut self) -> TokenKind {
        if self.peek() == Some('=') {
            self.advance();
            TokenKind::PercentEqual
        } else {
            TokenKind::Percent
        }
    }

    fn scan_less_than(&mut self) -> TokenKind {
        match self.peek() {
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LeftShiftEqual
                } else {
                    TokenKind::LeftShift
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::LessThanEqual
            }
            _ => TokenKind::LessThan,
        }
    }

    fn scan_greater_than(&mut self) -> TokenKind {
        match self.peek() {
            Some('>') => {
                self.advance();
                match self.peek() {
                    Some('>') => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            TokenKind::UnsignedRightShiftEqual
                        } else {
                            TokenKind::UnsignedRightShift
                        }
                    }
                    Some('=') => {
                        self.advance();
                        TokenKind::RightShiftEqual
                    }
                    _ => TokenKind::RightShift,
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::GreaterThanEqual
            }
            _ => TokenKind::GreaterThan,
        }
    }

    fn scan_equal(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StrictEqual
                } else {
                    TokenKind::EqualEqual
                }
            }
            Some('>') => {
                self.advance();
                TokenKind::Arrow
            }
            _ => TokenKind::Equal,
        }
    }

    fn scan_bang(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StrictNotEqual
                } else {
                    TokenKind::NotEqual
                }
            }
            _ => TokenKind::Bang,
        }
    }

    fn scan_ampersand(&mut self) -> TokenKind {
        match self.peek() {
            Some('&') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::AmpersandAmpersandEqual
                } else {
                    TokenKind::AmpersandAmpersand
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::AmpersandEqual
            }
            _ => TokenKind::Ampersand,
        }
    }

    fn scan_pipe(&mut self) -> TokenKind {
        match self.peek() {
            Some('|') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PipePipeEqual
                } else {
                    TokenKind::PipePipe
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::PipeEqual
            }
            _ => TokenKind::Pipe,
        }
    }

    fn scan_caret(&mut self) -> TokenKind {
        if self.peek() == Some('=') {
            self.advance();
            TokenKind::CaretEqual
        } else {
            TokenKind::Caret
        }
    }

    fn scan_question(&mut self) -> TokenKind {
        match self.peek() {
            Some('?') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::QuestionQuestionEqual
                } else {
                    TokenKind::QuestionQuestion
                }
            }
            Some('.') => {
                self.advance();
                TokenKind::QuestionDot
            }
            _ => TokenKind::Question,
        }
    }

    fn scan_string(&mut self, quote: char) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Invalid, // Unterminated string
                Some((_, ch)) if ch == quote => break,
                Some((_, '\\')) => {
                    // Handle escape sequences
                    if let Some((_, escaped)) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            '\\' => value.push('\\'),
                            '\'' => value.push('\''),
                            '"' => value.push('"'),
                            '0' => value.push('\0'),
                            // TODO: Unicode escapes, hex escapes
                            _ => value.push(escaped),
                        }
                    }
                }
                Some((_, ch)) => value.push(ch),
            }
        }

        TokenKind::String(value)
    }

    fn scan_template(&mut self) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Invalid, // Unterminated template
                Some((_, '`')) => break,
                Some((_, '$')) if self.peek() == Some('{') => {
                    // TODO: Handle template expressions
                    self.advance();
                    return TokenKind::Template(value);
                }
                Some((_, '\\')) => {
                    if let Some((_, escaped)) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            '\\' => value.push('\\'),
                            '`' => value.push('`'),
                            '$' => value.push('$'),
                            _ => value.push(escaped),
                        }
                    }
                }
                Some((_, ch)) => value.push(ch),
            }
        }

        TokenKind::Template(value)
    }

    fn scan_number(&mut self, first: char) -> TokenKind {
        let mut value = String::from(first);

        // Handle hex, octal, binary
        if first == '0' {
            match self.peek() {
                Some('x' | 'X') => return self.scan_hex_number(),
                Some('o' | 'O') => return self.scan_octal_number(),
                Some('b' | 'B') => return self.scan_binary_number(),
                _ => {}
            }
        }

        // Integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        if self.peek() == Some('.') {
            value.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    if ch != '_' {
                        value.push(ch);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if matches!(self.peek(), Some('e' | 'E')) {
            value.push('e');
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                value.push(self.advance().unwrap().1);
            }
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // BigInt suffix
        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(value);
        }

        match value.parse::<f64>() {
            Ok(n) => TokenKind::Number(n),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_hex_number(&mut self) -> TokenKind {
        self.advance(); // consume 'x'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_hexdigit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0x{}", value));
        }

        match u64::from_str_radix(&value, 16) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_octal_number(&mut self) -> TokenKind {
        self.advance(); // consume 'o'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_digit(8) || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0o{}", value));
        }

        match u64::from_str_radix(&value, 8) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_binary_number(&mut self) -> TokenKind {
        self.advance(); // consume 'b'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '0' || ch == '1' || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0b{}", value));
        }

        match u64::from_str_radix(&value, 2) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_identifier(&mut self, first: char) -> TokenKind {
        let mut name = String::from(first);

        while let Some(ch) = self.peek() {
            if is_id_continue(ch) {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for JavaScript keywords first
        match name.as_str() {
            "await" => TokenKind::Await,
            "break" => TokenKind::Break,
            "case" => TokenKind::Case,
            "catch" => TokenKind::Catch,
            "class" => TokenKind::Class,
            "const" => TokenKind::Const,
            "continue" => TokenKind::Continue,
            "debugger" => TokenKind::Debugger,
            "default" => TokenKind::Default,
            "delete" => TokenKind::Delete,
            "do" => TokenKind::Do,
            "else" => TokenKind::Else,
            "enum" => TokenKind::Enum,
            "export" => TokenKind::Export,
            "extends" => TokenKind::Extends,
            "false" => TokenKind::False,
            "finally" => TokenKind::Finally,
            "for" => TokenKind::For,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "import" => TokenKind::Import,
            "in" => TokenKind::In,
            "instanceof" => TokenKind::Instanceof,
            "let" => TokenKind::Let,
            "new" => TokenKind::New,
            "null" => TokenKind::Null,
            "return" => TokenKind::Return,
            "static" => TokenKind::Static,
            "super" => TokenKind::Super,
            "switch" => TokenKind::Switch,
            "this" => TokenKind::This,
            "throw" => TokenKind::Throw,
            "true" => TokenKind::True,
            "try" => TokenKind::Try,
            "typeof" => TokenKind::Typeof,
            "var" => TokenKind::Var,
            "void" => TokenKind::Void,
            "while" => TokenKind::While,
            "with" => TokenKind::With,
            "yield" => TokenKind::Yield,
            "async" => TokenKind::Async,
            // TypeScript keywords (only recognized in TypeScript mode)
            "type" if self.typescript_mode => TokenKind::Type,
            "interface" if self.typescript_mode => TokenKind::Interface,
            "namespace" if self.typescript_mode => TokenKind::Namespace,
            "declare" if self.typescript_mode => TokenKind::Declare,
            "readonly" if self.typescript_mode => TokenKind::Readonly,
            "abstract" if self.typescript_mode => TokenKind::Abstract,
            "implements" if self.typescript_mode => TokenKind::Implements,
            "private" if self.typescript_mode => TokenKind::Private,
            "protected" if self.typescript_mode => TokenKind::Protected,
            "public" if self.typescript_mode => TokenKind::Public,
            "as" if self.typescript_mode => TokenKind::As,
            "is" if self.typescript_mode => TokenKind::Is,
            "keyof" if self.typescript_mode => TokenKind::Keyof,
            "infer" if self.typescript_mode => TokenKind::Infer,
            "never" if self.typescript_mode => TokenKind::Never,
            "unknown" if self.typescript_mode => TokenKind::Unknown,
            "any" if self.typescript_mode => TokenKind::Any,
            "asserts" if self.typescript_mode => TokenKind::Asserts,
            "override" if self.typescript_mode => TokenKind::Override,
            "satisfies" if self.typescript_mode => TokenKind::Satisfies,
            "out" if self.typescript_mode => TokenKind::Out,
            "accessor" if self.typescript_mode => TokenKind::Accessor,
            _ => TokenKind::Identifier(name),
        }
    }

    fn scan_private_identifier(&mut self) -> TokenKind {
        let mut name = String::new();

        while let Some(ch) = self.peek() {
            if is_id_continue(ch) {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if name.is_empty() {
            TokenKind::Invalid
        } else {
            TokenKind::PrivateIdentifier(name)
        }
    }
}

/// Checks if a character can start an identifier.
fn is_id_start(ch: char) -> bool {
    ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_start(ch)
}

/// Checks if a character can continue an identifier.
fn is_id_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_continue(ch)
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut scanner = Scanner::new("{ } ( )");
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftParen));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightParen));
    }

    #[test]
    fn test_all_single_char_tokens() {
        let mut scanner = Scanner::new("{}()[];,~:");
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftParen));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightParen));
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftBracket));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightBracket));
        assert!(matches!(scanner.next_token().kind, TokenKind::Semicolon));
        assert!(matches!(scanner.next_token().kind, TokenKind::Comma));
        assert!(matches!(scanner.next_token().kind, TokenKind::Tilde));
        assert!(matches!(scanner.next_token().kind, TokenKind::Colon));
    }

    #[test]
    fn test_numbers() {
        let mut scanner = Scanner::new("42 3.14 0xff 0b1010");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 3.14));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 255.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 10.0));
    }

    #[test]
    fn test_octal_numbers() {
        let mut scanner = Scanner::new("0o777 0O10");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 511.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 8.0));
    }

    #[test]
    fn test_bigint() {
        let mut scanner = Scanner::new("123n 0xFFn 0o77n 0b1010n");
        assert!(matches!(scanner.next_token().kind, TokenKind::BigInt(s) if s == "123"));
        assert!(matches!(scanner.next_token().kind, TokenKind::BigInt(s) if s == "0xFF"));
        assert!(matches!(scanner.next_token().kind, TokenKind::BigInt(s) if s == "0o77"));
        assert!(matches!(scanner.next_token().kind, TokenKind::BigInt(s) if s == "0b1010"));
    }

    #[test]
    fn test_numbers_with_exponent() {
        let mut scanner = Scanner::new("1e10 2.5e-3 3E+4");
        assert!(
            matches!(scanner.next_token().kind, TokenKind::Number(n) if (n - 1e10).abs() < 0.001)
        );
        assert!(
            matches!(scanner.next_token().kind, TokenKind::Number(n) if (n - 2.5e-3).abs() < 0.0000001)
        );
        assert!(
            matches!(scanner.next_token().kind, TokenKind::Number(n) if (n - 3e4).abs() < 0.001)
        );
    }

    #[test]
    fn test_numbers_with_underscores() {
        let mut scanner = Scanner::new("1_000_000 0xFF_FF");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 1_000_000.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 65535.0));
    }

    #[test]
    fn test_strings() {
        let mut scanner = Scanner::new(r#""hello" 'world'"#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "hello"));
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "world"));
    }

    #[test]
    fn test_string_escapes() {
        let mut scanner = Scanner::new(r#""\n\r\t\\\'\"\0""#);
        let token = scanner.next_token();
        if let TokenKind::String(s) = token.kind {
            assert_eq!(s, "\n\r\t\\'\"\0");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_template_literal() {
        let mut scanner = Scanner::new("`hello world`");
        assert!(matches!(scanner.next_token().kind, TokenKind::Template(s) if s == "hello world"));
    }

    #[test]
    fn test_template_with_escapes() {
        let mut scanner = Scanner::new(r#"`\n\t\`\$`"#);
        let token = scanner.next_token();
        if let TokenKind::Template(s) = token.kind {
            assert_eq!(s, "\n\t`$");
        } else {
            panic!("Expected template");
        }
    }

    #[test]
    fn test_keywords() {
        let mut scanner = Scanner::new("function const let var");
        assert!(matches!(scanner.next_token().kind, TokenKind::Function));
        assert!(matches!(scanner.next_token().kind, TokenKind::Const));
        assert!(matches!(scanner.next_token().kind, TokenKind::Let));
        assert!(matches!(scanner.next_token().kind, TokenKind::Var));
    }

    #[test]
    fn test_all_keywords() {
        let keywords = "await break case catch class const continue debugger default delete do else enum export extends finally for function if import in instanceof let new return static super switch this throw try typeof var void while with yield async true false null";
        let mut scanner = Scanner::new(keywords);

        let expected = vec![
            TokenKind::Await,
            TokenKind::Break,
            TokenKind::Case,
            TokenKind::Catch,
            TokenKind::Class,
            TokenKind::Const,
            TokenKind::Continue,
            TokenKind::Debugger,
            TokenKind::Default,
            TokenKind::Delete,
            TokenKind::Do,
            TokenKind::Else,
            TokenKind::Enum,
            TokenKind::Export,
            TokenKind::Extends,
            TokenKind::Finally,
            TokenKind::For,
            TokenKind::Function,
            TokenKind::If,
            TokenKind::Import,
            TokenKind::In,
            TokenKind::Instanceof,
            TokenKind::Let,
            TokenKind::New,
            TokenKind::Return,
            TokenKind::Static,
            TokenKind::Super,
            TokenKind::Switch,
            TokenKind::This,
            TokenKind::Throw,
            TokenKind::Try,
            TokenKind::Typeof,
            TokenKind::Var,
            TokenKind::Void,
            TokenKind::While,
            TokenKind::With,
            TokenKind::Yield,
            TokenKind::Async,
            TokenKind::True,
            TokenKind::False,
            TokenKind::Null,
        ];

        for expected_kind in expected {
            assert_eq!(scanner.next_token().kind, expected_kind);
        }
    }

    #[test]
    fn test_identifiers() {
        let mut scanner = Scanner::new("foo _bar $baz");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "_bar"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "$baz"));
    }

    #[test]
    fn test_private_identifier() {
        let mut scanner = Scanner::new("#privateField");
        assert!(
            matches!(scanner.next_token().kind, TokenKind::PrivateIdentifier(s) if s == "privateField")
        );
    }

    #[test]
    fn test_invalid_private_identifier() {
        let mut scanner = Scanner::new("# ");
        assert!(matches!(scanner.next_token().kind, TokenKind::Invalid));
    }

    #[test]
    fn test_single_line_comments() {
        let mut scanner = Scanner::new("42 // this is a comment\n43");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 43.0));
    }

    #[test]
    fn test_multi_line_comments() {
        let mut scanner = Scanner::new("1 /* comment */ 2 /* multi\nline\ncomment */ 3");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 1.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 2.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 3.0));
    }

    #[test]
    fn test_division_vs_comment() {
        let mut scanner = Scanner::new("6 / 2");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 6.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Slash));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 2.0));
    }

    #[test]
    fn test_dot_operators() {
        let mut scanner = Scanner::new(". ... ..");
        assert!(matches!(scanner.next_token().kind, TokenKind::Dot));
        assert!(matches!(scanner.next_token().kind, TokenKind::Ellipsis));
        assert!(matches!(scanner.next_token().kind, TokenKind::Invalid)); // ".." is invalid
    }

    #[test]
    fn test_plus_operators() {
        let mut scanner = Scanner::new("+ ++ +=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Plus));
        assert!(matches!(scanner.next_token().kind, TokenKind::PlusPlus));
        assert!(matches!(scanner.next_token().kind, TokenKind::PlusEqual));
    }

    #[test]
    fn test_minus_operators() {
        let mut scanner = Scanner::new("- -- -=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Minus));
        assert!(matches!(scanner.next_token().kind, TokenKind::MinusMinus));
        assert!(matches!(scanner.next_token().kind, TokenKind::MinusEqual));
    }

    #[test]
    fn test_star_operators() {
        let mut scanner = Scanner::new("* ** *= **=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Star));
        assert!(matches!(scanner.next_token().kind, TokenKind::StarStar));
        assert!(matches!(scanner.next_token().kind, TokenKind::StarEqual));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::StarStarEqual
        ));
    }

    #[test]
    fn test_slash_operators() {
        let mut scanner = Scanner::new("/ /=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Slash));
        assert!(matches!(scanner.next_token().kind, TokenKind::SlashEqual));
    }

    #[test]
    fn test_percent_operators() {
        let mut scanner = Scanner::new("% %=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Percent));
        assert!(matches!(scanner.next_token().kind, TokenKind::PercentEqual));
    }

    #[test]
    fn test_less_than_operators() {
        let mut scanner = Scanner::new("< <= << <<=");
        assert!(matches!(scanner.next_token().kind, TokenKind::LessThan));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::LessThanEqual
        ));
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftShift));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::LeftShiftEqual
        ));
    }

    #[test]
    fn test_greater_than_operators() {
        let mut scanner = Scanner::new("> >= >> >>= >>> >>>=");
        assert!(matches!(scanner.next_token().kind, TokenKind::GreaterThan));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::GreaterThanEqual
        ));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightShift));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::RightShiftEqual
        ));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::UnsignedRightShift
        ));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::UnsignedRightShiftEqual
        ));
    }

    #[test]
    fn test_equal_operators() {
        let mut scanner = Scanner::new("= == === =>");
        assert!(matches!(scanner.next_token().kind, TokenKind::Equal));
        assert!(matches!(scanner.next_token().kind, TokenKind::EqualEqual));
        assert!(matches!(scanner.next_token().kind, TokenKind::StrictEqual));
        assert!(matches!(scanner.next_token().kind, TokenKind::Arrow));
    }

    #[test]
    fn test_bang_operators() {
        let mut scanner = Scanner::new("! != !==");
        assert!(matches!(scanner.next_token().kind, TokenKind::Bang));
        assert!(matches!(scanner.next_token().kind, TokenKind::NotEqual));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::StrictNotEqual
        ));
    }

    #[test]
    fn test_ampersand_operators() {
        let mut scanner = Scanner::new("& && &= &&=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Ampersand));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::AmpersandAmpersand
        ));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::AmpersandEqual
        ));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::AmpersandAmpersandEqual
        ));
    }

    #[test]
    fn test_pipe_operators() {
        let mut scanner = Scanner::new("| || |= ||=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Pipe));
        assert!(matches!(scanner.next_token().kind, TokenKind::PipePipe));
        assert!(matches!(scanner.next_token().kind, TokenKind::PipeEqual));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::PipePipeEqual
        ));
    }

    #[test]
    fn test_caret_operators() {
        let mut scanner = Scanner::new("^ ^=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Caret));
        assert!(matches!(scanner.next_token().kind, TokenKind::CaretEqual));
    }

    #[test]
    fn test_question_operators() {
        let mut scanner = Scanner::new("? ?? ?. ??=");
        assert!(matches!(scanner.next_token().kind, TokenKind::Question));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::QuestionQuestion
        ));
        assert!(matches!(scanner.next_token().kind, TokenKind::QuestionDot));
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::QuestionQuestionEqual
        ));
    }

    #[test]
    fn test_eof() {
        let mut scanner = Scanner::new("");
        assert!(matches!(scanner.next_token().kind, TokenKind::Eof));
    }

    #[test]
    fn test_whitespace_handling() {
        let mut scanner = Scanner::new("  \t\n\r  42  ");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
    }

    #[test]
    fn test_iterator() {
        let scanner = Scanner::new("1 + 2");
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Number(_)));
        assert!(matches!(tokens[1].kind, TokenKind::Plus));
        assert!(matches!(tokens[2].kind, TokenKind::Number(_)));
    }

    #[test]
    fn test_invalid_character() {
        // Use a character that's not valid in JS/TS (backslash at start)
        let mut scanner = Scanner::new("\\");
        assert!(matches!(scanner.next_token().kind, TokenKind::Invalid));
    }

    #[test]
    fn test_at_decorator_token() {
        let mut scanner = Scanner::new("@");
        assert!(matches!(scanner.next_token().kind, TokenKind::At));
    }

    #[test]
    fn test_span_tracking() {
        let mut scanner = Scanner::new("foo");
        let token = scanner.next_token();
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 3);
    }

    #[test]
    fn test_unicode_identifier() {
        let mut scanner = Scanner::new("π café 日本語");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "π"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "café"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "日本語"));
    }

    #[test]
    fn test_template_with_expression() {
        let mut scanner = Scanner::new("`hello ${");
        let token = scanner.next_token();
        assert!(matches!(token.kind, TokenKind::Template(s) if s == "hello "));
    }

    #[test]
    fn test_unterminated_string() {
        let mut scanner = Scanner::new("\"hello");
        assert!(matches!(scanner.next_token().kind, TokenKind::Invalid));
    }

    #[test]
    fn test_unterminated_template() {
        let mut scanner = Scanner::new("`hello");
        assert!(matches!(scanner.next_token().kind, TokenKind::Invalid));
    }

    #[test]
    fn test_comment_at_end() {
        let mut scanner = Scanner::new("42 // comment");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Eof));
    }

    #[test]
    fn test_complex_expression() {
        // const x = a + b * (c - d) / e;
        // const, x, =, a, +, b, *, (, c, -, d, ), /, e, ; = 15 tokens
        let scanner = Scanner::new("const x = a + b * (c - d) / e;");
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(tokens.len(), 15);
    }

    // TypeScript-specific tests

    #[test]
    fn test_typescript_keywords_in_js_mode() {
        // In JS mode, TypeScript keywords should be treated as identifiers
        let mut scanner = Scanner::new("type interface as");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "type"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "interface"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "as"));
    }

    #[test]
    fn test_typescript_keywords_in_ts_mode() {
        // In TS mode, TypeScript keywords should be recognized
        let mut scanner = Scanner::new_typescript("type interface as");
        assert!(matches!(scanner.next_token().kind, TokenKind::Type));
        assert!(matches!(scanner.next_token().kind, TokenKind::Interface));
        assert!(matches!(scanner.next_token().kind, TokenKind::As));
    }

    #[test]
    fn test_typescript_mode_toggle() {
        let mut scanner = Scanner::new("type");
        assert!(!scanner.is_typescript_mode());
        assert!(matches!(
            scanner.next_token().kind,
            TokenKind::Identifier(_)
        ));

        let mut scanner = Scanner::new_typescript("type");
        assert!(scanner.is_typescript_mode());
        assert!(matches!(scanner.next_token().kind, TokenKind::Type));
    }

    #[test]
    fn test_all_typescript_keywords() {
        let keywords = "type interface namespace declare readonly abstract implements private protected public as is keyof infer never unknown any asserts override satisfies out accessor";
        let mut scanner = Scanner::new_typescript(keywords);

        assert!(matches!(scanner.next_token().kind, TokenKind::Type));
        assert!(matches!(scanner.next_token().kind, TokenKind::Interface));
        assert!(matches!(scanner.next_token().kind, TokenKind::Namespace));
        assert!(matches!(scanner.next_token().kind, TokenKind::Declare));
        assert!(matches!(scanner.next_token().kind, TokenKind::Readonly));
        assert!(matches!(scanner.next_token().kind, TokenKind::Abstract));
        assert!(matches!(scanner.next_token().kind, TokenKind::Implements));
        assert!(matches!(scanner.next_token().kind, TokenKind::Private));
        assert!(matches!(scanner.next_token().kind, TokenKind::Protected));
        assert!(matches!(scanner.next_token().kind, TokenKind::Public));
        assert!(matches!(scanner.next_token().kind, TokenKind::As));
        assert!(matches!(scanner.next_token().kind, TokenKind::Is));
        assert!(matches!(scanner.next_token().kind, TokenKind::Keyof));
        assert!(matches!(scanner.next_token().kind, TokenKind::Infer));
        assert!(matches!(scanner.next_token().kind, TokenKind::Never));
        assert!(matches!(scanner.next_token().kind, TokenKind::Unknown));
        assert!(matches!(scanner.next_token().kind, TokenKind::Any));
        assert!(matches!(scanner.next_token().kind, TokenKind::Asserts));
        assert!(matches!(scanner.next_token().kind, TokenKind::Override));
        assert!(matches!(scanner.next_token().kind, TokenKind::Satisfies));
        assert!(matches!(scanner.next_token().kind, TokenKind::Out));
        assert!(matches!(scanner.next_token().kind, TokenKind::Accessor));
    }
}
