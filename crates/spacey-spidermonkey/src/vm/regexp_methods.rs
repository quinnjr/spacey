//! RegExp object method implementations.

use crate::runtime::value::Value;

/// Call a regexp method
pub fn call_regexp_method(regex_str: &str, method: &str, args: &[Value]) -> Value {
    // Parse the regex string (format: /pattern/flags)
    let (pattern, flags) = parse_regexp_string(regex_str);

    match method {
        "test" => {
            let input = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let result = simple_regex_match(&pattern, &input, &flags).is_some();
            Value::Boolean(result)
        }
        "exec" => {
            let input = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            match simple_regex_match(&pattern, &input, &flags) {
                Some((_start, matched)) => Value::String(matched),
                None => Value::Null,
            }
        }
        "toString" => {
            Value::String(regex_str.to_string())
        }
        _ => Value::Undefined,
    }
}

/// Parse a regexp string like "/pattern/flags" into (pattern, flags).
pub fn parse_regexp_string(s: &str) -> (String, String) {
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
pub fn simple_regex_match(pattern: &str, input: &str, flags: &str) -> Option<(usize, String)> {
    let ignore_case = flags.contains('i');

    // Handle empty pattern
    if pattern.is_empty() {
        return Some((0, String::new()));
    }

    // Very basic pattern matching
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
        "^" => Some((0, String::new())),
        "$" => Some((input.len(), String::new())),
        "." => {
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

/// Replace first match in a string
pub fn simple_regex_replace(pattern: &str, input: &str, replacement: &str, flags: &str) -> String {
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
        format!("{}{}{}", before, replacement, after)
    } else {
        input.to_string()
    }
}

/// Replace all matches in a string
pub fn simple_regex_replace_all(pattern: &str, input: &str, replacement: &str, flags: &str) -> String {
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
        result.push_str(replacement);
        start = abs_idx + pattern.len().max(1);
    }

    result.push_str(&input[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_regexp_string() {
        let (pattern, flags) = parse_regexp_string("/hello/gi");
        assert_eq!(pattern, "hello");
        assert_eq!(flags, "gi");

        let (pattern, flags) = parse_regexp_string("/test/");
        assert_eq!(pattern, "test");
        assert_eq!(flags, "");

        let (pattern, flags) = parse_regexp_string("hello");
        assert_eq!(pattern, "hello");
        assert_eq!(flags, "");
    }

    #[test]
    fn test_simple_regex_match() {
        let result = simple_regex_match("hello", "hello world", "");
        assert_eq!(result, Some((0, "hello".to_string())));

        let result = simple_regex_match("world", "hello world", "");
        assert_eq!(result, Some((6, "world".to_string())));

        let result = simple_regex_match("xyz", "hello world", "");
        assert_eq!(result, None);
    }

    #[test]
    fn test_simple_regex_match_case_insensitive() {
        let result = simple_regex_match("HELLO", "hello world", "i");
        assert_eq!(result, Some((0, "hello".to_string())));
    }

    #[test]
    fn test_simple_regex_replace() {
        let result = simple_regex_replace("world", "hello world", "rust", "");
        assert_eq!(result, "hello rust");

        let result = simple_regex_replace("xyz", "hello world", "rust", "");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_simple_regex_replace_all() {
        let result = simple_regex_replace_all("o", "hello world", "0", "");
        assert_eq!(result, "hell0 w0rld");
    }

    #[test]
    fn test_call_regexp_method_test() {
        let result = call_regexp_method("/hello/", "test", &[Value::String("hello world".into())]);
        assert!(matches!(result, Value::Boolean(true)));

        let result = call_regexp_method("/xyz/", "test", &[Value::String("hello world".into())]);
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_call_regexp_method_exec() {
        let result = call_regexp_method("/hello/", "exec", &[Value::String("hello world".into())]);
        assert!(matches!(result, Value::String(s) if s == "hello"));

        let result = call_regexp_method("/xyz/", "exec", &[Value::String("hello world".into())]);
        assert!(matches!(result, Value::Null));
    }
}



