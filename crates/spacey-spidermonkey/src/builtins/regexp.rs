//! RegExp built-in object (ES3 Section 15.10).
//!
//! Provides RegExp constructor and prototype methods.

use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::function::{CallFrame, Callable};
use crate::runtime::value::Value;

// ============================================================================
// RegExp Constructor (ES3 Section 15.10.3-4)
// ============================================================================

/// RegExp() called as function or constructor.
///
/// ES3 Section 15.10.3-4
pub fn regexp_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let pattern = args
        .first()
        .map(|v| match v {
            Value::Undefined => String::new(),
            _ => v.to_js_string(),
        })
        .unwrap_or_default();

    let flags = args
        .get(1)
        .map(|v| match v {
            Value::Undefined => String::new(),
            _ => v.to_js_string(),
        })
        .unwrap_or_default();

    // Validate flags
    let mut global = false;
    let mut ignore_case = false;
    let mut multiline = false;

    for c in flags.chars() {
        match c {
            'g' => {
                if global {
                    return Err("SyntaxError: invalid regular expression flags".to_string());
                }
                global = true;
            }
            'i' => {
                if ignore_case {
                    return Err("SyntaxError: invalid regular expression flags".to_string());
                }
                ignore_case = true;
            }
            'm' => {
                if multiline {
                    return Err("SyntaxError: invalid regular expression flags".to_string());
                }
                multiline = true;
            }
            _ => {
                return Err("SyntaxError: invalid regular expression flags".to_string());
            }
        }
    }

    // Create a RegExp object with properties and methods
    let mut regexp_obj = HashMap::new();

    // Store the regex string representation for internal use
    let regex_str = format!("/{}/{}", pattern, flags);
    regexp_obj.insert("__regex__".to_string(), Value::String(regex_str.clone()));
    regexp_obj.insert("__type__".to_string(), Value::String("RegExp".to_string()));

    // Instance properties
    regexp_obj.insert("source".to_string(), Value::String(pattern));
    regexp_obj.insert("global".to_string(), Value::Boolean(global));
    regexp_obj.insert("ignoreCase".to_string(), Value::Boolean(ignore_case));
    regexp_obj.insert("multiline".to_string(), Value::Boolean(multiline));
    regexp_obj.insert("lastIndex".to_string(), Value::Number(0.0));

    // Methods as native functions
    regexp_obj.insert(
        "test".to_string(),
        Value::Function(Arc::new(Callable::Native {
            name: "RegExp.prototype.test".to_string(),
            arity: 1,
            func: test,
        })),
    );
    regexp_obj.insert(
        "exec".to_string(),
        Value::Function(Arc::new(Callable::Native {
            name: "RegExp.prototype.exec".to_string(),
            arity: 1,
            func: exec,
        })),
    );
    regexp_obj.insert(
        "toString".to_string(),
        Value::Function(Arc::new(Callable::Native {
            name: "RegExp.prototype.toString".to_string(),
            arity: 0,
            func: to_string,
        })),
    );

    Ok(Value::NativeObject(regexp_obj))
}

// ============================================================================
// RegExp.prototype Methods (ES3 Section 15.10.6)
// ============================================================================

/// RegExp.prototype.exec(string) - Execute a search.
///
/// ES3 Section 15.10.6.2
pub fn exec(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // First arg is 'this' (the regexp), second is the input string
    let regexp_str = get_regexp_string(args.first());
    let input = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

    // Parse the regexp string (format: /pattern/flags)
    let (pattern, flags) = parse_regexp_string(&regexp_str);

    // Try to match using basic pattern matching
    match simple_regex_match(&pattern, &input, &flags) {
        Some((_start, matched)) => {
            // In full impl, would return an array with match info
            // For now, return the matched string
            Ok(Value::String(matched))
        }
        None => Ok(Value::Null),
    }
}

/// RegExp.prototype.test(string) - Test for a match.
///
/// ES3 Section 15.10.6.3
pub fn test(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // First arg is 'this' (the regexp), second is the input string
    let regexp_str = get_regexp_string(args.first());
    let input = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

    let (pattern, flags) = parse_regexp_string(&regexp_str);

    let result = simple_regex_match(&pattern, &input, &flags).is_some();
    Ok(Value::Boolean(result))
}

/// RegExp.prototype.toString() - Returns regex as string.
///
/// ES3 Section 15.10.6.4
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let regexp_str = get_regexp_string(args.first());

    // If already in /pattern/flags format, return as-is
    if regexp_str.starts_with('/') {
        return Ok(Value::String(regexp_str));
    }

    // Otherwise, wrap in slashes
    Ok(Value::String(format!("/{}/", regexp_str)))
}

/// Get the regex string from a value (handles both NativeObject and String).
fn get_regexp_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::NativeObject(props)) => {
            // Get the __regex__ property from the RegExp object
            props
                .get("__regex__")
                .map(|v| v.to_js_string())
                .unwrap_or_default()
        }
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_js_string(),
        None => String::new(),
    }
}

// ============================================================================
// RegExp Instance Properties
// ============================================================================

/// Get RegExp.prototype.source
pub fn get_source(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let regexp_str = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let (pattern, _) = parse_regexp_string(&regexp_str);
    Ok(Value::String(pattern))
}

/// Get RegExp.prototype.global
pub fn get_global(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let regexp_str = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let (_, flags) = parse_regexp_string(&regexp_str);
    Ok(Value::Boolean(flags.contains('g')))
}

/// Get RegExp.prototype.ignoreCase
pub fn get_ignore_case(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let regexp_str = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let (_, flags) = parse_regexp_string(&regexp_str);
    Ok(Value::Boolean(flags.contains('i')))
}

/// Get RegExp.prototype.multiline
pub fn get_multiline(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let regexp_str = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let (_, flags) = parse_regexp_string(&regexp_str);
    Ok(Value::Boolean(flags.contains('m')))
}

/// Get RegExp.prototype.lastIndex
pub fn get_last_index(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // In a full impl, this would be stored on the RegExp object
    Ok(Value::Number(0.0))
}

/// Set RegExp.prototype.lastIndex
pub fn set_last_index(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // In a full impl, this would update the RegExp object
    Ok(Value::Undefined)
}

// ============================================================================
// String Methods That Use RegExp (ES3 Section 15.5.4)
// ============================================================================

/// String.prototype.match(regexp) - implemented here for regex support.
pub fn string_match(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let string = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let regexp_str = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

    let (pattern, flags) = parse_regexp_string(&regexp_str);
    let global = flags.contains('g');

    if global {
        // Global: return array of all matches
        let matches = simple_regex_match_all(&pattern, &string, &flags);
        if matches.is_empty() {
            Ok(Value::Null)
        } else {
            // Return as comma-separated string for now
            Ok(Value::String(matches.join(",")))
        }
    } else {
        // Non-global: return first match or null
        match simple_regex_match(&pattern, &string, &flags) {
            Some((_, matched)) => Ok(Value::String(matched)),
            None => Ok(Value::Null),
        }
    }
}

/// String.prototype.replace(searchValue, replaceValue)
pub fn string_replace(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let string = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let search = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
    let replace = args.get(2).map(|v| v.to_js_string()).unwrap_or_default();

    let (pattern, flags) = parse_regexp_string(&search);
    let global = flags.contains('g');

    if pattern.is_empty() && search.is_empty() {
        return Ok(Value::String(string));
    }

    if global {
        // Global replace
        let result = simple_regex_replace_all(&pattern, &string, &replace, &flags);
        Ok(Value::String(result))
    } else {
        // Replace first match only
        let result = simple_regex_replace(&pattern, &string, &replace, &flags);
        Ok(Value::String(result))
    }
}

/// String.prototype.search(regexp)
pub fn string_search(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let string = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let regexp_str = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

    let (pattern, flags) = parse_regexp_string(&regexp_str);

    match simple_regex_match(&pattern, &string, &flags) {
        Some((index, _)) => Ok(Value::Number(index as f64)),
        None => Ok(Value::Number(-1.0)),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse a regexp string like "/pattern/flags" into (pattern, flags).
fn parse_regexp_string(s: &str) -> (String, String) {
    if s.starts_with('/') {
        // Find the last '/' to separate pattern from flags
        if let Some(last_slash) = s.rfind('/')
            && last_slash > 0 {
                let pattern = s[1..last_slash].to_string();
                let flags = s[last_slash + 1..].to_string();
                return (pattern, flags);
            }
    }
    // Not in /pattern/flags format, treat entire string as pattern
    (s.to_string(), String::new())
}

/// Simple regex matching (basic implementation without full regex crate).
/// Returns (start_index, matched_string) or None.
fn simple_regex_match(pattern: &str, input: &str, flags: &str) -> Option<(usize, String)> {
    let ignore_case = flags.contains('i');

    // Handle empty pattern
    if pattern.is_empty() {
        return Some((0, String::new()));
    }

    // Very basic pattern matching
    // In a full implementation, would use the regex crate
    let search_input = if ignore_case {
        input.to_lowercase()
    } else {
        input.to_string()
    };

    let search_pattern = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    // Handle special regex patterns (simplified)
    match search_pattern.as_str() {
        "^" => {
            // Match start of string
            Some((0, String::new()))
        }
        "$" => {
            // Match end of string
            Some((input.len(), String::new()))
        }
        "." => {
            // Match any single character
            if !input.is_empty() {
                Some((0, input.chars().next().unwrap().to_string()))
            } else {
                None
            }
        }
        _ => {
            // Literal string search
            if let Some(idx) = search_input.find(&search_pattern) {
                let matched = &input[idx..idx + pattern.len()];
                Some((idx, matched.to_string()))
            } else {
                None
            }
        }
    }
}

/// Find all matches (simplified).
fn simple_regex_match_all(pattern: &str, input: &str, flags: &str) -> Vec<String> {
    let ignore_case = flags.contains('i');
    let mut matches = Vec::new();

    if pattern.is_empty() {
        return matches;
    }

    let search_input = if ignore_case {
        input.to_lowercase()
    } else {
        input.to_string()
    };

    let search_pattern = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    let mut start = 0;
    while let Some(idx) = search_input[start..].find(&search_pattern) {
        let abs_idx = start + idx;
        let matched = &input[abs_idx..abs_idx + pattern.len()];
        matches.push(matched.to_string());
        start = abs_idx + pattern.len().max(1);
        if start >= input.len() {
            break;
        }
    }

    matches
}

/// Replace first match (simplified).
fn simple_regex_replace(pattern: &str, input: &str, replacement: &str, flags: &str) -> String {
    let ignore_case = flags.contains('i');

    if pattern.is_empty() {
        return format!("{}{}", replacement, input);
    }

    let search_input = if ignore_case {
        input.to_lowercase()
    } else {
        input.to_string()
    };

    let search_pattern = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    if let Some(idx) = search_input.find(&search_pattern) {
        let before = &input[..idx];
        let after = &input[idx + pattern.len()..];
        // Handle special replacement patterns
        let repl = process_replacement(replacement, &input[idx..idx + pattern.len()]);
        format!("{}{}{}", before, repl, after)
    } else {
        input.to_string()
    }
}

/// Replace all matches (simplified).
fn simple_regex_replace_all(pattern: &str, input: &str, replacement: &str, flags: &str) -> String {
    let ignore_case = flags.contains('i');

    if pattern.is_empty() {
        // Empty pattern: insert replacement at each position
        let chars: Vec<char> = input.chars().collect();
        let mut result = replacement.to_string();
        for c in chars {
            result.push(c);
            result.push_str(replacement);
        }
        return result;
    }

    let search_input = if ignore_case {
        input.to_lowercase()
    } else {
        input.to_string()
    };

    let search_pattern = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    let mut result = String::new();
    let mut start = 0;

    while let Some(idx) = search_input[start..].find(&search_pattern) {
        let abs_idx = start + idx;
        result.push_str(&input[start..abs_idx]);
        let matched = &input[abs_idx..abs_idx + pattern.len()];
        let repl = process_replacement(replacement, matched);
        result.push_str(&repl);
        start = abs_idx + pattern.len().max(1);
    }

    result.push_str(&input[start..]);
    result
}

/// Process replacement string for special patterns like $&, $1, etc.
fn process_replacement(replacement: &str, matched: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = replacement.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            match chars[i + 1] {
                '$' => {
                    result.push('$');
                    i += 2;
                }
                '&' => {
                    result.push_str(matched);
                    i += 2;
                }
                '`' => {
                    // $` - portion before match (not implemented)
                    i += 2;
                }
                '\'' => {
                    // $' - portion after match (not implemented)
                    i += 2;
                }
                _ => {
                    result.push(chars[i]);
                    i += 1;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
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

    // ========================================================================
    // Constructor Tests
    // ========================================================================

    #[test]
    fn test_regexp_constructor_empty() {
        let mut frame = make_frame();
        let result = regexp_constructor(&mut frame, &[]).unwrap();
        // RegExp now returns a NativeObject
        if let Value::NativeObject(props) = result {
            assert_eq!(props.get("__regex__"), Some(&Value::String("//".to_string())));
        } else {
            panic!("Expected NativeObject, got {:?}", result);
        }
    }

    #[test]
    fn test_regexp_constructor_pattern_only() {
        let mut frame = make_frame();
        let result = regexp_constructor(&mut frame, &[Value::String("test".to_string())]).unwrap();
        // RegExp now returns a NativeObject
        if let Value::NativeObject(props) = result {
            assert_eq!(props.get("__regex__"), Some(&Value::String("/test/".to_string())));
        } else {
            panic!("Expected NativeObject, got {:?}", result);
        }
    }

    #[test]
    fn test_regexp_constructor_with_flags() {
        let mut frame = make_frame();
        let result = regexp_constructor(
            &mut frame,
            &[
                Value::String("test".to_string()),
                Value::String("gi".to_string()),
            ],
        )
        .unwrap();
        // RegExp now returns a NativeObject
        if let Value::NativeObject(props) = result {
            assert_eq!(props.get("__regex__"), Some(&Value::String("/test/gi".to_string())));
        } else {
            panic!("Expected NativeObject, got {:?}", result);
        }
    }

    #[test]
    fn test_regexp_constructor_invalid_flags() {
        let mut frame = make_frame();
        let result = regexp_constructor(
            &mut frame,
            &[
                Value::String("test".to_string()),
                Value::String("x".to_string()),
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_regexp_constructor_duplicate_flags() {
        let mut frame = make_frame();
        let result = regexp_constructor(
            &mut frame,
            &[
                Value::String("test".to_string()),
                Value::String("gg".to_string()),
            ],
        );
        assert!(result.is_err());
    }

    // ========================================================================
    // test() Method Tests
    // ========================================================================

    #[test]
    fn test_regexp_test_match() {
        let mut frame = make_frame();
        let result = test(
            &mut frame,
            &[
                Value::String("/hello/".to_string()),
                Value::String("hello world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_regexp_test_no_match() {
        let mut frame = make_frame();
        let result = test(
            &mut frame,
            &[
                Value::String("/xyz/".to_string()),
                Value::String("hello world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_regexp_test_case_insensitive() {
        let mut frame = make_frame();
        let result = test(
            &mut frame,
            &[
                Value::String("/HELLO/i".to_string()),
                Value::String("hello world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    // ========================================================================
    // exec() Method Tests
    // ========================================================================

    #[test]
    fn test_regexp_exec_match() {
        let mut frame = make_frame();
        let result = exec(
            &mut frame,
            &[
                Value::String("/world/".to_string()),
                Value::String("hello world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "world"));
    }

    #[test]
    fn test_regexp_exec_no_match() {
        let mut frame = make_frame();
        let result = exec(
            &mut frame,
            &[
                Value::String("/xyz/".to_string()),
                Value::String("hello world".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Null));
    }

    // ========================================================================
    // toString() Tests
    // ========================================================================

    #[test]
    fn test_regexp_to_string() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::String("/test/gi".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "/test/gi"));
    }

    // ========================================================================
    // Property Getter Tests
    // ========================================================================

    #[test]
    fn test_get_source() {
        let mut frame = make_frame();
        let result = get_source(&mut frame, &[Value::String("/hello/gi".to_string())]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_get_global() {
        let mut frame = make_frame();
        let result = get_global(&mut frame, &[Value::String("/test/g".to_string())]).unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        let result = get_global(&mut frame, &[Value::String("/test/".to_string())]).unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_get_ignore_case() {
        let mut frame = make_frame();
        let result = get_ignore_case(&mut frame, &[Value::String("/test/i".to_string())]).unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_get_multiline() {
        let mut frame = make_frame();
        let result = get_multiline(&mut frame, &[Value::String("/test/m".to_string())]).unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    // ========================================================================
    // String Method Tests
    // ========================================================================

    #[test]
    fn test_string_match_found() {
        let mut frame = make_frame();
        let result = string_match(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/world/".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "world"));
    }

    #[test]
    fn test_string_match_not_found() {
        let mut frame = make_frame();
        let result = string_match(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/xyz/".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_string_match_global() {
        let mut frame = make_frame();
        let result = string_match(
            &mut frame,
            &[
                Value::String("hello hello hello".to_string()),
                Value::String("/hello/g".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s.contains("hello")));
    }

    #[test]
    fn test_string_replace_single() {
        let mut frame = make_frame();
        let result = string_replace(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/world/".to_string()),
                Value::String("universe".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello universe"));
    }

    #[test]
    fn test_string_replace_global() {
        let mut frame = make_frame();
        let result = string_replace(
            &mut frame,
            &[
                Value::String("hello hello hello".to_string()),
                Value::String("/hello/g".to_string()),
                Value::String("hi".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hi hi hi"));
    }

    #[test]
    fn test_string_replace_special_pattern() {
        let mut frame = make_frame();
        let result = string_replace(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/world/".to_string()),
                Value::String("$&!".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello world!"));
    }

    #[test]
    fn test_string_search_found() {
        let mut frame = make_frame();
        let result = string_search(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/world/".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 6.0));
    }

    #[test]
    fn test_string_search_not_found() {
        let mut frame = make_frame();
        let result = string_search(
            &mut frame,
            &[
                Value::String("hello world".to_string()),
                Value::String("/xyz/".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0));
    }

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    #[test]
    fn test_parse_regexp_string() {
        let (pattern, flags) = parse_regexp_string("/hello/gi");
        assert_eq!(pattern, "hello");
        assert_eq!(flags, "gi");

        let (pattern, flags) = parse_regexp_string("test");
        assert_eq!(pattern, "test");
        assert_eq!(flags, "");

        let (pattern, flags) = parse_regexp_string("/test/");
        assert_eq!(pattern, "test");
        assert_eq!(flags, "");
    }

    #[test]
    fn test_process_replacement() {
        assert_eq!(process_replacement("foo", "match"), "foo");
        assert_eq!(process_replacement("$&", "match"), "match");
        assert_eq!(process_replacement("$$", "match"), "$");
        assert_eq!(process_replacement("a$&b", "X"), "aXb");
    }
}
