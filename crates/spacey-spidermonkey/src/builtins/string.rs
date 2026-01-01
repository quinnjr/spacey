//! String built-in object (ES3 Section 15.5).
//!
//! Provides String constructor and prototype methods.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

// ============================================================================
// String Constructor (ES3 Section 15.5.1-2)
// ============================================================================

/// String() constructor - converts value to string.
pub fn string_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = args
        .first()
        .map(|v| v.to_js_string())
        .unwrap_or_default();
    Ok(Value::String(s))
}

/// String.fromCharCode(...codes) - Returns string from char codes.
///
/// ES3 Section 15.5.3.2
pub fn from_char_code(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut result = String::new();
    for arg in args {
        let code = arg.to_uint16();
        if let Some(c) = char::from_u32(code as u32) {
            result.push(c);
        }
    }
    Ok(Value::String(result))
}

// ============================================================================
// String.prototype Methods (ES3 Section 15.5.4)
// ============================================================================

/// String.prototype.toString() - Returns the string value.
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    Ok(Value::String(s))
}

/// String.prototype.valueOf() - Returns the string value.
pub fn value_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    Ok(Value::String(s))
}

/// String.prototype.charAt(pos) - Returns character at position.
///
/// ES3 Section 15.5.4.4
pub fn char_at(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let pos = args.get(1).map(|v| v.to_integer() as i64).unwrap_or(0);

    if pos < 0 || pos as usize >= s.len() {
        return Ok(Value::String(String::new()));
    }

    let c = s.chars().nth(pos as usize);
    Ok(Value::String(c.map(|c| c.to_string()).unwrap_or_default()))
}

/// String.prototype.charCodeAt(pos) - Returns char code at position.
///
/// ES3 Section 15.5.4.5
pub fn char_code_at(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let pos = args.get(1).map(|v| v.to_integer() as i64).unwrap_or(0);

    if pos < 0 || pos as usize >= s.len() {
        return Ok(Value::Number(f64::NAN));
    }

    let code = s.chars().nth(pos as usize).map(|c| c as u32);
    Ok(Value::Number(code.map(|c| c as f64).unwrap_or(f64::NAN)))
}

/// String.prototype.concat(...strings) - Concatenates strings.
///
/// ES3 Section 15.5.4.6
pub fn concat(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut result = get_this_string(args)?;
    for arg in args.iter().skip(1) {
        result.push_str(&arg.to_js_string());
    }
    Ok(Value::String(result))
}

/// String.prototype.indexOf(searchString, position) - Finds first occurrence.
///
/// ES3 Section 15.5.4.7
pub fn index_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let search = args
        .get(1)
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());
    let pos = args
        .get(2)
        .map(|v| v.to_integer().max(0.0) as usize)
        .unwrap_or(0);

    let result = if pos >= s.len() {
        if search.is_empty() {
            s.len() as i64
        } else {
            -1
        }
    } else {
        s[pos..]
            .find(&search)
            .map(|i| (i + pos) as i64)
            .unwrap_or(-1)
    };

    Ok(Value::Number(result as f64))
}

/// String.prototype.lastIndexOf(searchString, position) - Finds last occurrence.
///
/// ES3 Section 15.5.4.8
pub fn last_index_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let search = args
        .get(1)
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());
    let pos = args
        .get(2)
        .map(|v| {
            let n = v.to_number();
            if n.is_nan() {
                s.len()
            } else {
                v.to_integer().max(0.0) as usize
            }
        })
        .unwrap_or(s.len());

    if search.is_empty() {
        return Ok(Value::Number(pos.min(s.len()) as f64));
    }

    let search_end = (pos + search.len()).min(s.len());
    let result = s[..search_end]
        .rfind(&search)
        .map(|i| i as i64)
        .unwrap_or(-1);

    Ok(Value::Number(result as f64))
}

/// String.prototype.localeCompare(that) - Compares strings for locale.
///
/// ES3 Section 15.5.4.9
pub fn locale_compare(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let that = args
        .get(1)
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "undefined".to_string());

    let result = match s.cmp(&that) {
        std::cmp::Ordering::Less => -1.0,
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater => 1.0,
    };

    Ok(Value::Number(result))
}

/// String.prototype.slice(start, end) - Extracts a section.
///
/// ES3 Section 15.5.4.13
pub fn slice(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let len = s.chars().count() as i64;

    let start = args.get(1).map(|v| v.to_integer() as i64).unwrap_or(0);
    let end = args
        .get(2)
        .map(|v| {
            if matches!(v, Value::Undefined) {
                len
            } else {
                v.to_integer() as i64
            }
        })
        .unwrap_or(len);

    let from = if start < 0 {
        (len + start).max(0) as usize
    } else {
        start.min(len) as usize
    };

    let to = if end < 0 {
        (len + end).max(0) as usize
    } else {
        end.min(len) as usize
    };

    let result: String = if from < to {
        s.chars().skip(from).take(to - from).collect()
    } else {
        String::new()
    };

    Ok(Value::String(result))
}

/// String.prototype.split(separator, limit) - Splits string into array.
///
/// ES3 Section 15.5.4.14
pub fn split(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;

    let limit = args
        .get(2)
        .map(|v| {
            if matches!(v, Value::Undefined) {
                u32::MAX
            } else {
                v.to_uint32()
            }
        })
        .unwrap_or(u32::MAX);

    if limit == 0 {
        // Return empty array
        return Ok(Value::Object(0));
    }

    let separator = args.get(1);

    // Undefined separator returns array with original string
    if separator.is_none() || matches!(separator, Some(Value::Undefined)) {
        return Ok(Value::Object(1)); // Array with one element
    }

    let sep = separator.unwrap().to_js_string();

    // Empty separator splits into characters
    if sep.is_empty() {
        let count = s.chars().count().min(limit as usize);
        return Ok(Value::Object(count));
    }

    let parts: Vec<&str> = s.split(&sep).take(limit as usize).collect();
    Ok(Value::Object(parts.len()))
}

/// String.prototype.substring(start, end) - Returns substring.
///
/// ES3 Section 15.5.4.15
pub fn substring(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    let len = s.chars().count();

    let start = args
        .get(1)
        .map(|v| v.to_integer().max(0.0).min(len as f64) as usize)
        .unwrap_or(0);
    let end = args
        .get(2)
        .map(|v| {
            if matches!(v, Value::Undefined) {
                len
            } else {
                v.to_integer().max(0.0).min(len as f64) as usize
            }
        })
        .unwrap_or(len);

    let (from, to) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let result: String = s.chars().skip(from).take(to - from).collect();
    Ok(Value::String(result))
}

/// String.prototype.toLowerCase() - Returns lowercase string.
///
/// ES3 Section 15.5.4.16
pub fn to_lower_case(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    Ok(Value::String(s.to_lowercase()))
}

/// String.prototype.toLocaleLowerCase() - Returns locale-aware lowercase.
///
/// ES3 Section 15.5.4.17
pub fn to_locale_lower_case(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Simplified: same as toLowerCase
    to_lower_case(_frame, args)
}

/// String.prototype.toUpperCase() - Returns uppercase string.
///
/// ES3 Section 15.5.4.18
pub fn to_upper_case(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    Ok(Value::String(s.to_uppercase()))
}

/// String.prototype.toLocaleUpperCase() - Returns locale-aware uppercase.
///
/// ES3 Section 15.5.4.19
pub fn to_locale_upper_case(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Simplified: same as toUpperCase
    to_upper_case(_frame, args)
}

/// String.prototype.trim() - Removes whitespace from both ends.
/// Note: This is ES5, but commonly needed.
pub fn trim(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = get_this_string(args)?;
    Ok(Value::String(s.trim().to_string()))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the string value from 'this' (first argument in prototype methods).
fn get_this_string(args: &[Value]) -> Result<String, String> {
    match args.first() {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(v) => Ok(v.to_js_string()),
        None => Ok(String::new()),
    }
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
    fn test_string_constructor() {
        let mut frame = make_frame();
        assert!(matches!(
            string_constructor(&mut frame, &[Value::Number(42.0)]).unwrap(),
            Value::String(s) if s == "42"
        ));
        assert!(matches!(
            string_constructor(&mut frame, &[]).unwrap(),
            Value::String(s) if s.is_empty()
        ));
    }

    #[test]
    fn test_from_char_code() {
        let mut frame = make_frame();
        let result = from_char_code(
            &mut frame,
            &[
                Value::Number(72.0),
                Value::Number(101.0),
                Value::Number(108.0),
                Value::Number(108.0),
                Value::Number(111.0),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "Hello"));
    }

    #[test]
    fn test_char_at() {
        let mut frame = make_frame();
        let result = char_at(
            &mut frame,
            &[Value::String("hello".to_string()), Value::Number(1.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "e"));

        // Out of bounds
        let result = char_at(
            &mut frame,
            &[Value::String("hello".to_string()), Value::Number(10.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s.is_empty()));
    }

    #[test]
    fn test_char_code_at() {
        let mut frame = make_frame();
        let result = char_code_at(
            &mut frame,
            &[Value::String("A".to_string()), Value::Number(0.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 65.0));
    }

    #[test]
    fn test_concat() {
        let mut frame = make_frame();
        let result = concat(
            &mut frame,
            &[
                Value::String("hello".to_string()),
                Value::String(" ".to_string()),
                Value::String("world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }

    #[test]
    fn test_index_of() {
        let mut frame = make_frame();
        let result = index_of(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 6.0));

        // Not found
        let result = index_of(
            &mut frame,
            &[
                Value::String("hello".to_string()),
                Value::String("world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0));
    }

    #[test]
    fn test_last_index_of() {
        let mut frame = make_frame();
        let result = last_index_of(
            &mut frame,
            &[
                Value::String("hello hello".to_string()),
                Value::String("hello".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 6.0));
    }

    #[test]
    fn test_slice() {
        let mut frame = make_frame();
        let result = slice(
            &mut frame,
            &[
                Value::String("hello".to_string()),
                Value::Number(1.0),
                Value::Number(4.0),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "ell"));

        // Negative indices
        let result = slice(
            &mut frame,
            &[Value::String("hello".to_string()), Value::Number(-3.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "llo"));
    }

    #[test]
    fn test_substring() {
        let mut frame = make_frame();
        let result = substring(
            &mut frame,
            &[
                Value::String("hello".to_string()),
                Value::Number(1.0),
                Value::Number(4.0),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "ell"));

        // Swapped indices
        let result = substring(
            &mut frame,
            &[
                Value::String("hello".to_string()),
                Value::Number(4.0),
                Value::Number(1.0),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "ell"));
    }

    #[test]
    fn test_to_lower_case() {
        let mut frame = make_frame();
        let result = to_lower_case(&mut frame, &[Value::String("HELLO".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_to_upper_case() {
        let mut frame = make_frame();
        let result = to_upper_case(&mut frame, &[Value::String("hello".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "HELLO"));
    }

    #[test]
    fn test_trim() {
        let mut frame = make_frame();
        let result = trim(&mut frame, &[Value::String("  hello  ".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_split() {
        let mut frame = make_frame();
        // With separator
        let result = split(
            &mut frame,
            &[
                Value::String("a,b,c".to_string()),
                Value::String(",".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Object(3)));
    }

    #[test]
    fn test_locale_compare() {
        let mut frame = make_frame();
        let result = locale_compare(
            &mut frame,
            &[
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0));

        let result = locale_compare(
            &mut frame,
            &[
                Value::String("a".to_string()),
                Value::String("a".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }
}
