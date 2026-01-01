//! Global built-in functions (ES3 Section 15.1).
//!
//! These are top-level functions available globally.

use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;
use crate::vm::VM;

// ============================================================================
// eval() (ES3 Section 15.1.2.1)
// ============================================================================

/// eval(x) - Evaluates JavaScript code represented as a string.
///
/// ES3 Section 15.1.2.1
pub fn eval(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let code = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            // If argument is not a string, return it unchanged
            return Ok(other.clone());
        }
        None => return Ok(Value::Undefined),
    };

    // Try parsing as-is first, if that fails try adding a semicolon
    let mut parser = Parser::new(&code);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(_) => {
            // Try adding a semicolon for bare expressions
            let code_with_semi = format!("{};", code.trim());
            let mut parser2 = Parser::new(&code_with_semi);
            parser2.parse_program().map_err(|e| format!("SyntaxError: {}", e))?
        }
    };

    // Compile the program
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&program)
        .map_err(|e| format!("SyntaxError: {}", e))?;

    // Execute the bytecode
    let mut vm = VM::new();
    vm.execute(&bytecode).map_err(|e| format!("Error: {}", e))
}

// ============================================================================
// parseInt, parseFloat, isNaN, isFinite (ES3 Section 15.1.2)
// ============================================================================

/// parseInt(string, radix) - Parses a string and returns an integer.
///
/// ES3 Section 15.1.2.2
pub fn parse_int(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    let radix = args.get(1).map(|v| v.to_int32()).unwrap_or(0);

    // Trim leading whitespace
    let s = input.trim_start();
    if s.is_empty() {
        return Ok(Value::Number(f64::NAN));
    }

    // Check for sign
    let (sign, s) = if let Some(stripped) = s.strip_prefix('-') {
        (-1.0, stripped)
    } else if let Some(stripped) = s.strip_prefix('+') {
        (1.0, stripped)
    } else {
        (1.0, s)
    };

    // Determine radix
    let radix = if radix == 0 {
        if s.starts_with("0x") || s.starts_with("0X") {
            16
        } else if s.starts_with('0') && s.len() > 1 {
            // ES3 allows octal with leading 0, ES5 doesn't
            8
        } else {
            10
        }
    } else if !(2..=36).contains(&radix) {
        return Ok(Value::Number(f64::NAN));
    } else {
        radix
    };

    // Strip 0x prefix for hex
    let s = if radix == 16 && (s.starts_with("0x") || s.starts_with("0X")) {
        &s[2..]
    } else {
        s
    };

    // Parse the number
    let mut result = 0.0;
    let mut found_digit = false;

    for c in s.chars() {
        let digit = match c {
            '0'..='9' => c as i32 - '0' as i32,
            'a'..='z' => c as i32 - 'a' as i32 + 10,
            'A'..='Z' => c as i32 - 'A' as i32 + 10,
            _ => break,
        };

        if digit >= radix {
            break;
        }

        found_digit = true;
        result = result * radix as f64 + digit as f64;
    }

    if !found_digit {
        return Ok(Value::Number(f64::NAN));
    }

    Ok(Value::Number(sign * result))
}

/// parseFloat(string) - Parses a string and returns a floating-point number.
///
/// ES3 Section 15.1.2.3
pub fn parse_float(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    let s = input.trim_start();
    if s.is_empty() {
        return Ok(Value::Number(f64::NAN));
    }

    // Handle Infinity
    if s.starts_with("Infinity") || s.starts_with("+Infinity") {
        return Ok(Value::Number(f64::INFINITY));
    }
    if s.starts_with("-Infinity") {
        return Ok(Value::Number(f64::NEG_INFINITY));
    }

    // Find the longest valid number prefix
    let mut end = 0;
    let mut _has_dot = false;
    let mut _has_exp = false;
    let chars: Vec<char> = s.chars().collect();

    // Optional sign
    if end < chars.len() && (chars[end] == '+' || chars[end] == '-') {
        end += 1;
    }

    // Digits before decimal
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }

    // Optional decimal point and digits
    if end < chars.len() && chars[end] == '.' {
        _has_dot = true;
        end += 1;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }

    // Optional exponent
    if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
        let exp_start = end;
        end += 1;

        // Optional sign
        if end < chars.len() && (chars[end] == '+' || chars[end] == '-') {
            end += 1;
        }

        // Must have at least one digit
        let digit_start = end;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }

        if end == digit_start {
            // No digits after e, back up
            end = exp_start;
        } else {
            _has_exp = true;
        }
    }

    if end == 0 || (end == 1 && (chars[0] == '+' || chars[0] == '-' || chars[0] == '.')) {
        return Ok(Value::Number(f64::NAN));
    }

    let num_str: String = chars[..end].iter().collect();
    match num_str.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n)),
        Err(_) => Ok(Value::Number(f64::NAN)),
    }
}

/// isNaN(number) - Returns true if the argument is NaN.
///
/// ES3 Section 15.1.2.4
pub fn is_nan(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let num = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Boolean(num.is_nan()))
}

/// isFinite(number) - Returns true if the argument is finite.
///
/// ES3 Section 15.1.2.5
pub fn is_finite(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let num = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Boolean(num.is_finite()))
}

/// encodeURI(uri) - Encodes a URI by replacing certain characters.
///
/// ES3 Section 15.1.3.1
pub fn encode_uri(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    // Characters that should NOT be encoded in encodeURI
    const RESERVED: &str = ";/?:@&=+$,#";
    const UNESCAPED: &str = "-_.!~*'()";
    const ALPHA_NUM: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let allowed: std::collections::HashSet<char> = RESERVED
        .chars()
        .chain(UNESCAPED.chars())
        .chain(ALPHA_NUM.chars())
        .collect();

    let mut result = String::new();
    for c in input.chars() {
        if allowed.contains(&c) {
            result.push(c);
        } else {
            // Encode as UTF-8
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }

    Ok(Value::String(result))
}

/// decodeURI(encodedURI) - Decodes a URI.
///
/// ES3 Section 15.1.3.2
pub fn decode_uri(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    // Characters that should NOT be decoded in decodeURI
    const RESERVED: &str = ";/?:@&=+$,#";

    decode_uri_component_internal(&input, RESERVED)
}

/// encodeURIComponent(str) - Encodes a URI component.
///
/// ES3 Section 15.1.3.3
pub fn encode_uri_component(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    // Characters that should NOT be encoded in encodeURIComponent
    const UNESCAPED: &str = "-_.!~*'()";
    const ALPHA_NUM: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let allowed: std::collections::HashSet<char> =
        UNESCAPED.chars().chain(ALPHA_NUM.chars()).collect();

    let mut result = String::new();
    for c in input.chars() {
        if allowed.contains(&c) {
            result.push(c);
        } else {
            // Encode as UTF-8
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }

    Ok(Value::String(result))
}

/// decodeURIComponent(encodedURIComponent) - Decodes a URI component.
///
/// ES3 Section 15.1.3.4
pub fn decode_uri_component(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let input = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    decode_uri_component_internal(&input, "")
}

/// Internal helper for URI decoding
fn decode_uri_component_internal(input: &str, reserved: &str) -> Result<Value, String> {
    let reserved_set: std::collections::HashSet<char> = reserved.chars().collect();
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' {
            if i + 2 >= chars.len() {
                return Err("URIError: malformed URI".to_string());
            }

            let hex: String = chars[i + 1..i + 3].iter().collect();
            let byte =
                u8::from_str_radix(&hex, 16).map_err(|_| "URIError: malformed URI".to_string())?;

            // Check if this is a reserved character
            let c = byte as char;
            if reserved_set.contains(&c) {
                result.push('%');
                result.push(chars[i + 1]);
                result.push(chars[i + 2]);
            } else {
                result.push(c);
            }
            i += 3;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Ok(Value::String(result))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::Function;

    fn make_frame() -> CallFrame {
        let func = Function::new(None, vec![], Bytecode::new(), 0);
        CallFrame::new(func, 0)
    }

    #[test]
    fn test_parse_int_decimal() {
        let mut frame = make_frame();
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("42".to_string())]).unwrap(),
            Value::Number(n) if n == 42.0
        ));
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("-42".to_string())]).unwrap(),
            Value::Number(n) if n == -42.0
        ));
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("  42  ".to_string())]).unwrap(),
            Value::Number(n) if n == 42.0
        ));
    }

    #[test]
    fn test_parse_int_hex() {
        let mut frame = make_frame();
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("0xFF".to_string())]).unwrap(),
            Value::Number(n) if n == 255.0
        ));
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("0x10".to_string())]).unwrap(),
            Value::Number(n) if n == 16.0
        ));
    }

    #[test]
    fn test_parse_int_with_radix() {
        let mut frame = make_frame();
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("1010".to_string()), Value::Number(2.0)]).unwrap(),
            Value::Number(n) if n == 10.0
        ));
        assert!(matches!(
            parse_int(&mut frame, &[Value::String("ff".to_string()), Value::Number(16.0)]).unwrap(),
            Value::Number(n) if n == 255.0
        ));
    }

    #[test]
    fn test_parse_int_nan() {
        let mut frame = make_frame();
        let result = parse_int(&mut frame, &[Value::String("abc".to_string())]).unwrap();
        assert!(matches!(result, Value::Number(n) if n.is_nan()));
    }

    #[test]
    fn test_parse_float() {
        let mut frame = make_frame();
        assert!(matches!(
            parse_float(&mut frame, &[Value::String("3.14".to_string())]).unwrap(),
            Value::Number(n) if (n - 3.14).abs() < 0.0001
        ));
        assert!(matches!(
            parse_float(&mut frame, &[Value::String("1e10".to_string())]).unwrap(),
            Value::Number(n) if (n - 1e10).abs() < 1.0
        ));
        assert!(matches!(
            parse_float(&mut frame, &[Value::String("Infinity".to_string())]).unwrap(),
            Value::Number(n) if n == f64::INFINITY
        ));
    }

    #[test]
    fn test_is_nan() {
        let mut frame = make_frame();
        assert!(matches!(
            is_nan(&mut frame, &[Value::Number(f64::NAN)]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            is_nan(&mut frame, &[Value::Number(42.0)]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            is_nan(&mut frame, &[Value::String("hello".to_string())]).unwrap(),
            Value::Boolean(true)
        ));
    }

    #[test]
    fn test_is_finite() {
        let mut frame = make_frame();
        assert!(matches!(
            is_finite(&mut frame, &[Value::Number(42.0)]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            is_finite(&mut frame, &[Value::Number(f64::INFINITY)]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            is_finite(&mut frame, &[Value::Number(f64::NAN)]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn test_encode_uri_component() {
        let mut frame = make_frame();
        let result =
            encode_uri_component(&mut frame, &[Value::String("hello world".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello%20world"));

        let result =
            encode_uri_component(&mut frame, &[Value::String("foo=bar".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "foo%3Dbar"));
    }

    #[test]
    fn test_decode_uri_component() {
        let mut frame = make_frame();
        let result =
            decode_uri_component(&mut frame, &[Value::String("hello%20world".to_string())])
                .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }
}
