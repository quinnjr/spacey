//! The scanner that produces tokens from source text.

use super::{Span, Token, TokenKind};

/// A scanner that tokenizes JavaScript source code.
pub struct Scanner<'a> {
    source: &'a str,
    /// Current position in the source string (public for parser lookahead)
    pub current_pos: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current_pos: 0,
        }
    }

    /// Returns the next token from the source.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.current_pos;

        let Some(ch) = self.advance() else {
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

            _ => TokenKind::Invalid,
        };

        Token::new(kind, Span::new(start, self.current_pos))
    }

    /// Scans a regex literal. This should be called by the parser when it knows
    /// a regex is expected (after certain tokens like `=`, `(`, etc.)
    pub fn scan_regexp(&mut self) -> Token {
        let start = self.current_pos;

        // The opening '/' has already been consumed as a Slash token
        // We need to back up and rescan
        // Actually, for ES5 compatibility, the parser will call this when it sees a Slash
        // and knows it should be a regex

        let mut pattern = String::new();
        let mut in_class = false; // inside [...]

        loop {
            match self.advance() {
                None => return Token::new(TokenKind::Invalid, Span::new(start, self.current_pos)),
                Some('/') if !in_class => break,
                Some('[') => {
                    in_class = true;
                    pattern.push('[');
                }
                Some(']') if in_class => {
                    in_class = false;
                    pattern.push(']');
                }
                Some('\\') => {
                    pattern.push('\\');
                    if let Some(ch) = self.advance() {
                        pattern.push(ch);
                    }
                }
                Some('\n') | Some('\r') => {
                    return Token::new(TokenKind::Invalid, Span::new(start, self.current_pos));
                }
                Some(ch) => pattern.push(ch),
            }
        }

        // Scan flags
        let mut flags = String::new();
        while let Some(ch) = self.peek() {
            if is_id_continue(ch) {
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

    fn advance(&mut self) -> Option<char> {
        if self.current_pos >= self.source.len() {
            return None;
        }
        let ch = self.source[self.current_pos..].chars().next()?;
        self.current_pos += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current_pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.current_pos..].chars();
        chars.next()?;
        chars.next()
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
                            // Single-line comment
                            self.advance(); // consume '/'
                            self.advance(); // consume second '/'
                            while let Some(ch) = self.peek() {
                                if ch == '\n' || ch == '\r' {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        Some('*') => {
                            // Multi-line comment
                            self.advance(); // consume '/'
                            self.advance(); // consume '*'
                            loop {
                                match self.advance() {
                                    None => break, // Unterminated comment
                                    Some('*') if self.peek() == Some('/') => {
                                        self.advance(); // consume '/'
                                        break;
                                    }
                                    _ => continue,
                                }
                            }
                        }
                        _ => {
                            // Division operator or regex - let the tokenizer handle it
                            break;
                        }
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
                Some(ch) if ch == quote => break,
                Some('\\') => {
                    // Handle escape sequences
                    if let Some(escaped) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            'b' => value.push('\x08'), // backspace
                            'f' => value.push('\x0C'), // form feed
                            'v' => value.push('\x0B'), // vertical tab
                            '\\' => value.push('\\'),
                            '\'' => value.push('\''),
                            '"' => value.push('"'),
                            '0' => {
                                // Check if it's a legacy octal or just \0
                                if let Some(next) = self.peek() {
                                    if next.is_ascii_digit() && next != '8' && next != '9' {
                                        // Legacy octal escape \0nn
                                        let mut octal = String::from("0");
                                        while let Some(ch) = self.peek() {
                                            if ch.is_digit(8) && octal.len() < 3 {
                                                octal.push(ch);
                                                self.advance();
                                            } else {
                                                break;
                                            }
                                        }
                                        if let Ok(code) = u32::from_str_radix(&octal, 8)
                                            && let Some(c) = char::from_u32(code)
                                        {
                                            value.push(c);
                                        }
                                    } else {
                                        value.push('\0');
                                    }
                                } else {
                                    value.push('\0');
                                }
                            }
                            '1'..='7' => {
                                // Legacy octal escape \nnn
                                let mut octal = String::from(escaped);
                                while let Some(ch) = self.peek() {
                                    if ch.is_digit(8) && octal.len() < 3 {
                                        octal.push(ch);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(code) = u32::from_str_radix(&octal, 8)
                                    && let Some(c) = char::from_u32(code)
                                {
                                    value.push(c);
                                }
                            }
                            'x' => {
                                // Hex escape \xHH
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if let Some(ch) = self.peek() {
                                        if ch.is_ascii_hexdigit() {
                                            hex.push(ch);
                                            self.advance();
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                if hex.len() == 2
                                    && let Ok(code) = u32::from_str_radix(&hex, 16)
                                    && let Some(c) = char::from_u32(code)
                                {
                                    value.push(c);
                                } else if hex.len() != 2 {
                                    // Invalid hex escape, push literal
                                    value.push('x');
                                    value.push_str(&hex);
                                }
                            }
                            'u' => {
                                // Unicode escape \uHHHH or \u{HHHH}
                                if self.peek() == Some('{') {
                                    // ES6 Unicode code point escape \u{HHHH}
                                    self.advance(); // consume '{'
                                    let mut hex = String::new();
                                    while let Some(ch) = self.peek() {
                                        if ch == '}' {
                                            self.advance();
                                            break;
                                        }
                                        if ch.is_ascii_hexdigit() {
                                            hex.push(ch);
                                            self.advance();
                                        } else {
                                            break;
                                        }
                                    }
                                    if let Ok(code) = u32::from_str_radix(&hex, 16)
                                        && let Some(c) = char::from_u32(code)
                                    {
                                        value.push(c);
                                    }
                                } else {
                                    // ES5 Unicode escape \uHHHH
                                    let mut hex = String::new();
                                    for _ in 0..4 {
                                        if let Some(ch) = self.peek() {
                                            if ch.is_ascii_hexdigit() {
                                                hex.push(ch);
                                                self.advance();
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                    if hex.len() == 4
                                        && let Ok(code) = u32::from_str_radix(&hex, 16)
                                        && let Some(c) = char::from_u32(code)
                                    {
                                        value.push(c);
                                    } else if hex.len() != 4 {
                                        // Invalid unicode escape, push literal
                                        value.push('u');
                                        value.push_str(&hex);
                                    }
                                }
                            }
                            '\n' => {
                                // Line continuation - skip the newline
                                // Also skip \r if present (Windows line ending)
                                if self.peek() == Some('\r') {
                                    self.advance();
                                }
                            }
                            '\r' => {
                                // Line continuation
                                if self.peek() == Some('\n') {
                                    self.advance();
                                }
                            }
                            _ => value.push(escaped),
                        }
                    }
                }
                Some(ch) => value.push(ch),
            }
        }

        TokenKind::String(value)
    }

    fn scan_template(&mut self) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Invalid, // Unterminated template
                Some('`') => break,
                Some('$') if self.peek() == Some('{') => {
                    // Template with substitution - return head part
                    self.advance(); // consume '{'
                    return TokenKind::TemplateHead(value);
                }
                Some('\\') => {
                    if let Some(escaped) = self.advance() {
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
                Some(ch) => value.push(ch),
            }
        }

        TokenKind::NoSubstitutionTemplate(value)
    }

    /// Scan the continuation of a template literal after a `}` (closing a substitution).
    /// This should be called by the parser when it has finished parsing an expression
    /// inside a template substitution.
    pub fn scan_template_continuation(&mut self) -> Token {
        let start = self.current_pos;
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return Token::new(TokenKind::Invalid, Span::new(start, self.current_pos)),
                Some('`') => {
                    // End of template
                    return Token::new(
                        TokenKind::TemplateTail(value),
                        Span::new(start, self.current_pos),
                    );
                }
                Some('$') if self.peek() == Some('{') => {
                    // Another substitution
                    self.advance(); // consume '{'
                    return Token::new(
                        TokenKind::TemplateMiddle(value),
                        Span::new(start, self.current_pos),
                    );
                }
                Some('\\') => {
                    if let Some(escaped) = self.advance() {
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
                Some(ch) => value.push(ch),
            }
        }
    }

    fn scan_number(&mut self, first: char) -> TokenKind {
        let mut value = String::from(first);

        // Handle hex, octal, binary
        if first == '0' {
            match self.peek() {
                Some('x' | 'X') => return self.scan_hex_number(),
                Some('o' | 'O') => return self.scan_octal_number(),
                Some('b' | 'B') => return self.scan_binary_number(),
                Some('0'..='7') => {
                    // Legacy octal literal (ES5 and earlier)
                    return self.scan_legacy_octal_number();
                }
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

        // Fractional part - but only if followed by a digit
        if self.peek() == Some('.') {
            // Check if next char after dot is a digit (to avoid confusion with member access)
            let has_fraction = self.peek_next().is_some_and(|c| c.is_ascii_digit());
            if has_fraction {
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
        }

        // Exponent part
        if matches!(self.peek(), Some('e' | 'E')) {
            value.push('e');
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                value.push(self.advance().unwrap());
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

    fn scan_legacy_octal_number(&mut self) -> TokenKind {
        // First digit already seen as '0', we're at the start of digits like 0777
        let mut value = String::new();
        let mut is_decimal = false;

        while let Some(ch) = self.peek() {
            if ch.is_digit(8) {
                value.push(ch);
                self.advance();
            } else if ch == '8' || ch == '9' {
                // This is actually a decimal number (invalid octal digit)
                is_decimal = true;
                value.push(ch);
                self.advance();
            } else if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if is_decimal {
            // Treat as decimal
            let full_value = format!("0{}", value);
            match full_value.parse::<f64>() {
                Ok(n) => TokenKind::Number(n),
                Err(_) => TokenKind::Invalid,
            }
        } else {
            // Parse as octal
            match u64::from_str_radix(&value, 8) {
                Ok(n) => TokenKind::Number(n as f64),
                Err(_) => TokenKind::Invalid,
            }
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

        // Check for keywords
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
    fn test_numbers() {
        let mut scanner = Scanner::new("42 3.14 0xff 0b1010");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 3.14));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 255.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 10.0));
    }

    #[test]
    fn test_legacy_octal() {
        let mut scanner = Scanner::new("0777 0644");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 511.0)); // 0777 octal = 511 decimal
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 420.0)); // 0644 octal = 420 decimal
    }

    #[test]
    fn test_strings() {
        let mut scanner = Scanner::new(r#""hello" 'world'"#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "hello"));
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "world"));
    }

    #[test]
    fn test_string_escapes() {
        let mut scanner = Scanner::new(r#""\n\t\r""#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "\n\t\r"));
    }

    #[test]
    fn test_unicode_escape() {
        let mut scanner = Scanner::new(r#""\u0041\u0042""#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "AB"));
    }

    #[test]
    fn test_hex_escape() {
        let mut scanner = Scanner::new(r#""\x41\x42""#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "AB"));
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
    fn test_identifiers() {
        let mut scanner = Scanner::new("foo _bar $baz");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "_bar"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "$baz"));
    }

    #[test]
    fn test_single_line_comment() {
        let mut scanner = Scanner::new("foo // this is a comment\nbar");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "bar"));
    }

    #[test]
    fn test_multi_line_comment() {
        let mut scanner = Scanner::new("foo /* this is\na comment */ bar");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "bar"));
    }

    #[test]
    fn test_division_not_comment() {
        let mut scanner = Scanner::new("a / b");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "a"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Slash));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "b"));
    }

    #[test]
    fn test_regex_literal() {
        let mut scanner = Scanner::new("/");
        scanner.next_token(); // consume initial slash
        scanner.current_pos = 0; // reset to test scan_regexp

        let mut scanner = Scanner::new("abc/gi");
        let token = scanner.scan_regexp();
        assert!(
            matches!(token.kind, TokenKind::RegExp { ref pattern, ref flags } if pattern == "abc" && flags == "gi")
        );
    }
}
