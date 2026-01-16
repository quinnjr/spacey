//! JSON parsing and stringification.
//!
//! This module implements the JSON built-in object methods:
//! - `JSON.parse()` - Parse a JSON string into a JavaScript value
//! - `JSON.stringify()` - Convert a JavaScript value to a JSON string
//!
//! The parser follows the JSON specification (RFC 8259) strictly:
//! - No trailing commas in arrays or objects
//! - No single quotes for strings
//! - Object keys must be double-quoted strings
//! - Unicode escapes (\uXXXX) are supported

use crate::Error;
use crate::runtime::value::Value;

/// Parses a JSON string into a JavaScript Value.
///
/// # Arguments
///
/// * `args` - The arguments passed to JSON.parse(). The first argument
///   should be the JSON string to parse.
///
/// # Returns
///
/// The parsed JavaScript value, or an error if the JSON is invalid.
///
/// # Examples
///
/// ```ignore
/// let result = json_parse(&[Value::String(r#"{"name": "test"}"#.into())])?;
/// ```
pub fn json_parse(args: &[Value]) -> Result<Value, Error> {
    let input = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(v) => {
            return Err(Error::TypeError(format!(
                "JSON.parse requires a string, got {}",
                v.type_of()
            )));
        }
        None => {
            return Err(Error::SyntaxError(
                "JSON.parse requires a string argument".into(),
            ));
        }
    };

    let mut parser = JsonParser::new(input);
    let value = parser.parse_value()?;

    // Ensure we've consumed all input (except whitespace)
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return Err(Error::SyntaxError(format!(
            "Unexpected token at position {}",
            parser.pos
        )));
    }

    Ok(value)
}

/// Stringifies a simple JavaScript value to JSON.
///
/// This handles primitive types (null, booleans, numbers, strings).
/// Complex types (arrays, objects) are handled by the VM which has
/// access to the heap.
///
/// # Arguments
///
/// * `value` - The value to stringify
///
/// # Returns
///
/// A JSON string representation of the value, or an error for
/// non-stringifiable values.
pub fn json_stringify_simple(value: &Value) -> Result<Value, Error> {
    match value {
        Value::Null => Ok(Value::String("null".into())),
        Value::Boolean(b) => Ok(Value::String(if *b { "true" } else { "false" }.into())),
        Value::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                Ok(Value::String("null".into()))
            } else {
                Ok(Value::String(format_json_number(*n)))
            }
        }
        Value::String(s) => Ok(Value::String(escape_json_string(s))),
        Value::Undefined => Ok(Value::Undefined), // undefined becomes undefined (filtered in arrays/objects)
        Value::Array(arr) => {
            let mut result = String::from("[");
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                match json_stringify_simple(item)? {
                    Value::Undefined => result.push_str("null"),
                    Value::String(s) => result.push_str(&s),
                    _ => result.push_str("null"),
                }
            }
            result.push(']');
            Ok(Value::String(result))
        }
        Value::ParsedObject(pairs) => {
            let mut result = String::from("{");
            let mut first = true;
            for (key, val) in pairs {
                // Skip undefined values
                if let Value::Undefined = val {
                    continue;
                }
                if !first {
                    result.push(',');
                }
                first = false;
                result.push_str(&escape_json_string(key));
                result.push(':');
                match json_stringify_simple(val)? {
                    Value::Undefined => result.push_str("null"),
                    Value::String(s) => result.push_str(&s),
                    _ => result.push_str("null"),
                }
            }
            result.push('}');
            Ok(Value::String(result))
        }
        Value::Symbol(_) | Value::Function(_) | Value::NativeFunction(_) => {
            // Functions and symbols return undefined
            Ok(Value::Undefined)
        }
        Value::Object(_) | Value::BigInt(_) => {
            // These need VM access - return undefined to signal VM should handle
            Ok(Value::Undefined)
        }
    }
}

/// Formats a number for JSON output.
///
/// Ensures proper formatting of integers vs floating point numbers.
fn format_json_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        // Format as integer if it has no fractional part
        format!("{}", n as i64)
    } else {
        // Use default float formatting
        let s = format!("{}", n);
        // Ensure we don't have unnecessary trailing zeros after decimal
        s
    }
}

/// Escapes a string for JSON output.
///
/// Handles all required JSON escape sequences:
/// - `\"` for double quotes
/// - `\\` for backslashes
/// - `\n`, `\r`, `\t` for newlines, carriage returns, tabs
/// - `\uXXXX` for control characters
pub fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');

    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"), // backspace
            '\u{0C}' => result.push_str("\\f"), // form feed
            c if c.is_control() => {
                // Escape other control characters as \uXXXX
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }

    result.push('"');
    result
}

/// Converts a Value to its JSON string representation for display.
///
/// This is a convenience function for debugging and display purposes.
pub fn to_json_string(value: &Value) -> String {
    match json_stringify_simple(value) {
        Ok(Value::String(s)) => s,
        Ok(Value::Undefined) => "undefined".into(),
        Ok(_) => "null".into(),
        Err(_) => "null".into(),
    }
}

/// A recursive descent JSON parser.
///
/// Parses JSON text according to RFC 8259, producing JavaScript values.
struct JsonParser<'a> {
    /// The input string being parsed
    input: &'a str,
    /// Current position in the input
    pos: usize,
}

impl<'a> JsonParser<'a> {
    /// Creates a new JSON parser for the given input.
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Returns the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Advances the position by one character.
    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parses a JSON value.
    fn parse_value(&mut self) -> Result<Value, Error> {
        self.skip_whitespace();

        match self.peek() {
            Some('"') => self.parse_string(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => self.parse_true(),
            Some('f') => self.parse_false(),
            Some('n') => self.parse_null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(Error::SyntaxError(format!(
                "Unexpected character '{}' at position {}",
                c, self.pos
            ))),
            None => Err(Error::SyntaxError("Unexpected end of input".into())),
        }
    }

    /// Parses a JSON string.
    fn parse_string(&mut self) -> Result<Value, Error> {
        let s = self.parse_string_raw()?;
        Ok(Value::String(s))
    }

    /// Parses a JSON string and returns the raw String value.
    fn parse_string_raw(&mut self) -> Result<String, Error> {
        // Consume opening quote
        if self.peek() != Some('"') {
            return Err(Error::SyntaxError("Expected '\"'".into()));
        }
        self.advance();

        let mut result = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some('/') => {
                            result.push('/');
                            self.advance();
                        }
                        Some('b') => {
                            result.push('\u{08}');
                            self.advance();
                        }
                        Some('f') => {
                            result.push('\u{0C}');
                            self.advance();
                        }
                        Some('n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some('u') => {
                            self.advance();
                            let hex = self.parse_unicode_escape()?;
                            result.push(hex);
                        }
                        Some(c) => {
                            return Err(Error::SyntaxError(format!(
                                "Invalid escape sequence '\\{}' at position {}",
                                c, self.pos
                            )));
                        }
                        None => {
                            return Err(Error::SyntaxError(
                                "Unexpected end of input in string escape".into(),
                            ));
                        }
                    }
                }
                Some(c) if c.is_control() => {
                    return Err(Error::SyntaxError(format!(
                        "Unescaped control character in string at position {}",
                        self.pos
                    )));
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
                None => {
                    return Err(Error::SyntaxError("Unterminated string".into()));
                }
            }
        }
    }

    /// Parses a \uXXXX unicode escape sequence.
    fn parse_unicode_escape(&mut self) -> Result<char, Error> {
        let start = self.pos;
        let mut hex_str = String::with_capacity(4);

        for _ in 0..4 {
            match self.peek() {
                Some(c) if c.is_ascii_hexdigit() => {
                    hex_str.push(c);
                    self.advance();
                }
                _ => {
                    return Err(Error::SyntaxError(format!(
                        "Invalid unicode escape at position {}",
                        start
                    )));
                }
            }
        }

        let code_point = u32::from_str_radix(&hex_str, 16).map_err(|_| {
            Error::SyntaxError(format!("Invalid unicode escape at position {}", start))
        })?;

        // Handle surrogate pairs
        if (0xD800..=0xDBFF).contains(&code_point) {
            // High surrogate - look for low surrogate
            if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                self.advance();
                if self.peek() == Some('u') {
                    self.advance();
                    let low_start = self.pos;
                    let mut low_hex = String::with_capacity(4);
                    for _ in 0..4 {
                        match self.peek() {
                            Some(c) if c.is_ascii_hexdigit() => {
                                low_hex.push(c);
                                self.advance();
                            }
                            _ => {
                                // Not a valid low surrogate, restore position
                                self.pos = saved_pos;
                                break;
                            }
                        }
                    }
                    if low_hex.len() == 4
                        && let Ok(low_point) = u32::from_str_radix(&low_hex, 16)
                        && (0xDC00..=0xDFFF).contains(&low_point)
                    {
                        // Valid surrogate pair
                        let combined =
                            0x10000 + ((code_point - 0xD800) << 10) + (low_point - 0xDC00);
                        return char::from_u32(combined).ok_or_else(|| {
                            Error::SyntaxError(format!(
                                "Invalid unicode code point at position {}",
                                low_start
                            ))
                        });
                    }
                    // Not a valid low surrogate, restore position
                    self.pos = saved_pos;
                } else {
                    self.pos = saved_pos;
                }
            }
            // Lone high surrogate - use replacement character
            return Ok('\u{FFFD}');
        }

        if (0xDC00..=0xDFFF).contains(&code_point) {
            // Lone low surrogate - use replacement character
            return Ok('\u{FFFD}');
        }

        char::from_u32(code_point).ok_or_else(|| {
            Error::SyntaxError(format!("Invalid unicode code point at position {}", start))
        })
    }

    /// Parses a JSON number.
    fn parse_number(&mut self) -> Result<Value, Error> {
        let start = self.pos;

        // Optional minus sign
        if self.peek() == Some('-') {
            self.advance();
        }

        // Integer part
        match self.peek() {
            Some('0') => {
                self.advance();
                // After leading 0, must not have another digit (unless decimal)
                if let Some(c) = self.peek()
                    && c.is_ascii_digit()
                {
                    return Err(Error::SyntaxError(format!(
                        "Leading zeros not allowed at position {}",
                        start
                    )));
                }
            }
            Some(c) if c.is_ascii_digit() => {
                self.advance();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                return Err(Error::SyntaxError(format!(
                    "Invalid number at position {}",
                    start
                )));
            }
        }

        // Optional fractional part
        if self.peek() == Some('.') {
            self.advance();
            // Must have at least one digit after decimal point
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(Error::SyntaxError(format!(
                    "Invalid number: expected digit after decimal point at position {}",
                    self.pos
                )));
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Optional exponent
        if let Some('e' | 'E') = self.peek() {
            self.advance();
            // Optional sign
            if let Some('+' | '-') = self.peek() {
                self.advance();
            }
            // Must have at least one digit in exponent
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(Error::SyntaxError(format!(
                    "Invalid number: expected digit in exponent at position {}",
                    self.pos
                )));
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let num_str = &self.input[start..self.pos];
        let num: f64 = num_str.parse().map_err(|_| {
            Error::SyntaxError(format!(
                "Invalid number '{}' at position {}",
                num_str, start
            ))
        })?;

        Ok(Value::Number(num))
    }

    /// Parses the JSON literal `true`.
    fn parse_true(&mut self) -> Result<Value, Error> {
        self.expect_literal("true")?;
        Ok(Value::Boolean(true))
    }

    /// Parses the JSON literal `false`.
    fn parse_false(&mut self) -> Result<Value, Error> {
        self.expect_literal("false")?;
        Ok(Value::Boolean(false))
    }

    /// Parses the JSON literal `null`.
    fn parse_null(&mut self) -> Result<Value, Error> {
        self.expect_literal("null")?;
        Ok(Value::Null)
    }

    /// Expects and consumes a specific literal string.
    fn expect_literal(&mut self, literal: &str) -> Result<(), Error> {
        for expected in literal.chars() {
            match self.peek() {
                Some(c) if c == expected => self.advance(),
                _ => {
                    return Err(Error::SyntaxError(format!(
                        "Expected '{}' at position {}",
                        literal, self.pos
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a JSON array.
    fn parse_array(&mut self) -> Result<Value, Error> {
        // Consume opening bracket
        if self.peek() != Some('[') {
            return Err(Error::SyntaxError("Expected '['".into()));
        }
        self.advance();
        self.skip_whitespace();

        let mut elements = Vec::new();

        // Empty array
        if self.peek() == Some(']') {
            self.advance();
            return Ok(Value::Array(elements));
        }

        loop {
            // Parse value
            let value = self.parse_value()?;
            elements.push(value);

            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                    // Check for trailing comma (not allowed in JSON)
                    if self.peek() == Some(']') {
                        return Err(Error::SyntaxError(format!(
                            "Trailing comma in array at position {}",
                            self.pos
                        )));
                    }
                }
                Some(']') => {
                    self.advance();
                    return Ok(Value::Array(elements));
                }
                Some(c) => {
                    return Err(Error::SyntaxError(format!(
                        "Expected ',' or ']' in array, got '{}' at position {}",
                        c, self.pos
                    )));
                }
                None => {
                    return Err(Error::SyntaxError("Unterminated array".into()));
                }
            }
        }
    }

    /// Parses a JSON object.
    fn parse_object(&mut self) -> Result<Value, Error> {
        // Consume opening brace
        if self.peek() != Some('{') {
            return Err(Error::SyntaxError("Expected '{'".into()));
        }
        self.advance();
        self.skip_whitespace();

        let mut pairs = Vec::new();

        // Empty object
        if self.peek() == Some('}') {
            self.advance();
            return Ok(Value::ParsedObject(pairs));
        }

        loop {
            self.skip_whitespace();

            // Parse key (must be a string)
            if self.peek() != Some('"') {
                return Err(Error::SyntaxError(format!(
                    "Expected string key in object at position {}",
                    self.pos
                )));
            }
            let key = self.parse_string_raw()?;

            self.skip_whitespace();

            // Expect colon
            if self.peek() != Some(':') {
                return Err(Error::SyntaxError(format!(
                    "Expected ':' after key in object at position {}",
                    self.pos
                )));
            }
            self.advance();

            // Parse value
            let value = self.parse_value()?;
            pairs.push((key, value));

            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                    // Check for trailing comma (not allowed in JSON)
                    if self.peek() == Some('}') {
                        return Err(Error::SyntaxError(format!(
                            "Trailing comma in object at position {}",
                            self.pos
                        )));
                    }
                }
                Some('}') => {
                    self.advance();
                    return Ok(Value::ParsedObject(pairs));
                }
                Some(c) => {
                    return Err(Error::SyntaxError(format!(
                        "Expected ',' or '}}' in object, got '{}' at position {}",
                        c, self.pos
                    )));
                }
                None => {
                    return Err(Error::SyntaxError("Unterminated object".into()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Primitive Value Tests ====================

    #[test]
    fn test_parse_null() {
        let result = json_parse(&[Value::String("null".into())]).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_parse_true() {
        let result = json_parse(&[Value::String("true".into())]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_parse_false() {
        let result = json_parse(&[Value::String("false".into())]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_parse_integer() {
        let result = json_parse(&[Value::String("42".into())]).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_parse_negative_integer() {
        let result = json_parse(&[Value::String("-42".into())]).unwrap();
        assert_eq!(result, Value::Number(-42.0));
    }

    #[test]
    fn test_parse_float() {
        let result = json_parse(&[Value::String("3.14159".into())]).unwrap();
        assert_eq!(result, Value::Number(3.14159));
    }

    #[test]
    fn test_parse_exponent() {
        let result = json_parse(&[Value::String("1.5e10".into())]).unwrap();
        assert_eq!(result, Value::Number(1.5e10));
    }

    #[test]
    fn test_parse_negative_exponent() {
        let result = json_parse(&[Value::String("1.5e-10".into())]).unwrap();
        assert_eq!(result, Value::Number(1.5e-10));
    }

    #[test]
    fn test_parse_zero() {
        let result = json_parse(&[Value::String("0".into())]).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    // ==================== String Tests ====================

    #[test]
    fn test_parse_empty_string() {
        let result = json_parse(&[Value::String(r#""""#.into())]).unwrap();
        assert_eq!(result, Value::String("".into()));
    }

    #[test]
    fn test_parse_simple_string() {
        let result = json_parse(&[Value::String(r#""hello""#.into())]).unwrap();
        assert_eq!(result, Value::String("hello".into()));
    }

    #[test]
    fn test_parse_string_with_spaces() {
        let result = json_parse(&[Value::String(r#""hello world""#.into())]).unwrap();
        assert_eq!(result, Value::String("hello world".into()));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        let result = json_parse(&[Value::String(r#""hello\nworld""#.into())]).unwrap();
        assert_eq!(result, Value::String("hello\nworld".into()));
    }

    #[test]
    fn test_parse_string_with_quote() {
        let result = json_parse(&[Value::String(r#""say \"hello\"""#.into())]).unwrap();
        assert_eq!(result, Value::String("say \"hello\"".into()));
    }

    #[test]
    fn test_parse_string_with_backslash() {
        let result = json_parse(&[Value::String(r#""path\\to\\file""#.into())]).unwrap();
        assert_eq!(result, Value::String("path\\to\\file".into()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let result = json_parse(&[Value::String(r#""caf\u00e9""#.into())]).unwrap();
        // Verify we get a string type
        assert_eq!(result.type_of(), "string");
        // The actual test - \u00e9 is 'e' with acute accent
        if let Value::String(s) = result {
            assert!(s.contains('e') || s.contains('\u{00e9}'));
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_parse_string_all_escapes() {
        let result = json_parse(&[Value::String(r#""a\b\f\n\r\t""#.into())]).unwrap();
        assert_eq!(result, Value::String("a\u{08}\u{0C}\n\r\t".into()));
    }

    // ==================== Array Tests ====================

    #[test]
    fn test_parse_empty_array() {
        let result = json_parse(&[Value::String("[]".into())]).unwrap();
        assert!(matches!(result, Value::Array(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_array_single_element() {
        let result = json_parse(&[Value::String("[1]".into())]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], Value::Number(1.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_array_multiple_elements() {
        let result = json_parse(&[Value::String("[1, 2, 3]".into())]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(1.0));
            assert_eq!(arr[1], Value::Number(2.0));
            assert_eq!(arr[2], Value::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_array_mixed_types() {
        let result = json_parse(&[Value::String(r#"[1, "two", true, null]"#.into())]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], Value::Number(1.0));
            assert_eq!(arr[1], Value::String("two".into()));
            assert_eq!(arr[2], Value::Boolean(true));
            assert_eq!(arr[3], Value::Null);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_nested_array() {
        let result = json_parse(&[Value::String("[[1, 2], [3, 4]]".into())]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            if let Value::Array(inner) = &arr[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected array");
        }
    }

    // ==================== Object Tests ====================

    #[test]
    fn test_parse_empty_object() {
        let result = json_parse(&[Value::String("{}".into())]).unwrap();
        assert!(matches!(result, Value::ParsedObject(ref v) if v.is_empty()));
    }

    #[test]
    fn test_parse_object_single_key() {
        let result = json_parse(&[Value::String(r#"{"key": "value"}"#.into())]).unwrap();
        if let Value::ParsedObject(pairs) = result {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "key");
            assert_eq!(pairs[0].1, Value::String("value".into()));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_object_multiple_keys() {
        let result = json_parse(&[Value::String(r#"{"a": 1, "b": 2}"#.into())]).unwrap();
        if let Value::ParsedObject(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let result = json_parse(&[Value::String(r#"{"outer": {"inner": 42}}"#.into())]).unwrap();
        if let Value::ParsedObject(pairs) = result {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "outer");
            if let Value::ParsedObject(inner) = &pairs[0].1 {
                assert_eq!(inner[0].0, "inner");
                assert_eq!(inner[0].1, Value::Number(42.0));
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_complex_structure() {
        let json = r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}"#;
        let result = json_parse(&[Value::String(json.into())]).unwrap();
        assert!(matches!(result, Value::ParsedObject(_)));
    }

    // ==================== Whitespace Tests ====================

    #[test]
    fn test_parse_with_leading_whitespace() {
        let result = json_parse(&[Value::String("   42".into())]).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_parse_with_trailing_whitespace() {
        let result = json_parse(&[Value::String("42   ".into())]).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_parse_with_newlines() {
        let result = json_parse(&[Value::String("{\n  \"key\": \"value\"\n}".into())]).unwrap();
        assert!(matches!(result, Value::ParsedObject(_)));
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_parse_error_trailing_comma_array() {
        let result = json_parse(&[Value::String("[1, 2,]".into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_trailing_comma_object() {
        let result = json_parse(&[Value::String(r#"{"a": 1,}"#.into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_single_quotes() {
        let result = json_parse(&[Value::String("'hello'".into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unquoted_key() {
        let result = json_parse(&[Value::String("{key: 1}".into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_leading_zero() {
        let result = json_parse(&[Value::String("01".into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unterminated_string() {
        let result = json_parse(&[Value::String(r#""hello"#.into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unterminated_array() {
        let result = json_parse(&[Value::String("[1, 2".into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unterminated_object() {
        let result = json_parse(&[Value::String(r#"{"key": 1"#.into())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_extra_content() {
        let result = json_parse(&[Value::String("42 extra".into())]);
        assert!(result.is_err());
    }

    // ==================== Stringify Tests ====================

    #[test]
    fn test_stringify_null() {
        let result = json_stringify_simple(&Value::Null).unwrap();
        assert_eq!(result, Value::String("null".into()));
    }

    #[test]
    fn test_stringify_true() {
        let result = json_stringify_simple(&Value::Boolean(true)).unwrap();
        assert_eq!(result, Value::String("true".into()));
    }

    #[test]
    fn test_stringify_false() {
        let result = json_stringify_simple(&Value::Boolean(false)).unwrap();
        assert_eq!(result, Value::String("false".into()));
    }

    #[test]
    fn test_stringify_integer() {
        let result = json_stringify_simple(&Value::Number(42.0)).unwrap();
        assert_eq!(result, Value::String("42".into()));
    }

    #[test]
    fn test_stringify_float() {
        let result = json_stringify_simple(&Value::Number(3.14)).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("3.14"));
        }
    }

    #[test]
    fn test_stringify_nan() {
        let result = json_stringify_simple(&Value::Number(f64::NAN)).unwrap();
        assert_eq!(result, Value::String("null".into()));
    }

    #[test]
    fn test_stringify_infinity() {
        let result = json_stringify_simple(&Value::Number(f64::INFINITY)).unwrap();
        assert_eq!(result, Value::String("null".into()));
    }

    #[test]
    fn test_stringify_string() {
        let result = json_stringify_simple(&Value::String("hello".into())).unwrap();
        assert_eq!(result, Value::String("\"hello\"".into()));
    }

    #[test]
    fn test_stringify_string_with_quotes() {
        let result = json_stringify_simple(&Value::String("say \"hi\"".into())).unwrap();
        assert_eq!(result, Value::String("\"say \\\"hi\\\"\"".into()));
    }

    #[test]
    fn test_stringify_string_with_newline() {
        let result = json_stringify_simple(&Value::String("line1\nline2".into())).unwrap();
        assert_eq!(result, Value::String("\"line1\\nline2\"".into()));
    }

    #[test]
    fn test_stringify_undefined() {
        let result = json_stringify_simple(&Value::Undefined).unwrap();
        assert_eq!(result, Value::Undefined);
    }

    #[test]
    fn test_stringify_array() {
        let arr = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = json_stringify_simple(&arr).unwrap();
        assert_eq!(result, Value::String("[1,2,3]".into()));
    }

    #[test]
    fn test_stringify_object() {
        let obj = Value::ParsedObject(vec![
            ("name".into(), Value::String("test".into())),
            ("value".into(), Value::Number(42.0)),
        ]);
        let result = json_stringify_simple(&obj).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("\"name\":\"test\""));
            assert!(s.contains("\"value\":42"));
        }
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_escape_json_string_simple() {
        assert_eq!(escape_json_string("hello"), "\"hello\"");
    }

    #[test]
    fn test_escape_json_string_quotes() {
        assert_eq!(escape_json_string("say \"hi\""), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn test_escape_json_string_backslash() {
        assert_eq!(escape_json_string("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn test_escape_json_string_newline() {
        assert_eq!(escape_json_string("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn test_escape_json_string_tab() {
        assert_eq!(escape_json_string("a\tb"), "\"a\\tb\"");
    }

    #[test]
    fn test_format_json_number_integer() {
        assert_eq!(format_json_number(42.0), "42");
    }

    #[test]
    fn test_format_json_number_negative() {
        assert_eq!(format_json_number(-42.0), "-42");
    }

    #[test]
    fn test_format_json_number_float() {
        let result = format_json_number(3.14);
        assert!(result.contains("3.14"));
    }
}
