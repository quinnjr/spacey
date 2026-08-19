//! String object method implementations.

use super::regexp_methods::{
    parse_regexp_string, simple_regex_match, simple_regex_replace, simple_regex_replace_all,
};
use crate::runtime::value::Value;

/// Call a string method
pub fn call_string_method(s: &str, method: &str, args: &[Value]) -> Value {
    match method {
        "charAt" => {
            let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            s.chars()
                .nth(idx)
                .map(|c| Value::String(c.to_string()))
                .unwrap_or(Value::String(String::new()))
        }
        "charCodeAt" => {
            let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            s.chars()
                .nth(idx)
                .map(|c| Value::Number(c as u32 as f64))
                .unwrap_or(Value::Number(f64::NAN))
        }
        "indexOf" => {
            let search = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let start = args.get(1).map(|v| v.to_number() as usize).unwrap_or(0);
            let search_str = s.get(start..).unwrap_or("");
            match search_str.find(&search) {
                Some(pos) => Value::Number((start + pos) as f64),
                None => Value::Number(-1.0),
            }
        }
        "lastIndexOf" => {
            let search = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            match s.rfind(&search) {
                Some(pos) => Value::Number(pos as f64),
                None => Value::Number(-1.0),
            }
        }
        "substring" => {
            let start = args.first().map(|v| v.to_number() as i32).unwrap_or(0);
            let end = args
                .get(1)
                .map(|v| v.to_number() as i32)
                .unwrap_or(s.len() as i32);
            let len = s.len() as i32;
            let start = start.max(0).min(len) as usize;
            let end = end.max(0).min(len) as usize;
            let (start, end) = if start > end {
                (end, start)
            } else {
                (start, end)
            };
            Value::String(s.chars().skip(start).take(end - start).collect())
        }
        "slice" => {
            let len = s.len() as i32;
            let start = args.first().map(|v| v.to_number() as i32).unwrap_or(0);
            let end = args.get(1).map(|v| v.to_number() as i32).unwrap_or(len);
            let start = if start < 0 {
                (len + start).max(0)
            } else {
                start.min(len)
            } as usize;
            let end = if end < 0 {
                (len + end).max(0)
            } else {
                end.min(len)
            } as usize;
            if start >= end {
                Value::String(String::new())
            } else {
                Value::String(s.chars().skip(start).take(end - start).collect())
            }
        }
        "substr" => {
            let start = args.first().map(|v| v.to_number() as i32).unwrap_or(0);
            let len_arg = args.get(1).map(|v| v.to_number() as i32);
            let s_len = s.len() as i32;
            let start = if start < 0 {
                (s_len + start).max(0)
            } else {
                start
            } as usize;
            let length = len_arg.unwrap_or(s_len - start as i32).max(0) as usize;
            Value::String(s.chars().skip(start).take(length).collect())
        }
        "toLowerCase" => Value::String(s.to_lowercase()),
        "toUpperCase" => Value::String(s.to_uppercase()),
        "split" => {
            let separator = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let parts: Vec<Value> = if separator.is_empty() {
                s.chars().map(|c| Value::String(c.to_string())).collect()
            } else {
                s.split(&separator)
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            };
            // Return as a simple object representing array (VM will handle creation)
            // For now, return a marker that the VM can process
            Value::String(format!(
                "__split_result__{}:{}",
                parts.len(),
                parts
                    .iter()
                    .map(|v| v.to_js_string())
                    .collect::<Vec<_>>()
                    .join("\x00")
            ))
        }
        "trim" => Value::String(s.trim().to_string()),
        "replace" => {
            let (search, is_regexp) = match args.first() {
                Some(Value::NativeObject(props)) => {
                    // RegExp object - extract __regex__ property
                    let regex_str = props
                        .get("__regex__")
                        .map(|v| v.to_js_string())
                        .unwrap_or_default();
                    (regex_str, true)
                }
                Some(v) => (v.to_js_string(), false),
                None => (String::new(), false),
            };
            let replacement = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

            // Check if search is a regexp string
            if is_regexp || (search.starts_with('/') && search.len() > 1) {
                let (pattern, flags) = parse_regexp_string(&search);
                if flags.contains('g') {
                    // Global replace
                    let result = simple_regex_replace_all(&pattern, s, &replacement, &flags);
                    return Value::String(result);
                } else {
                    // Replace first match
                    let result = simple_regex_replace(&pattern, s, &replacement, &flags);
                    return Value::String(result);
                }
            }

            // Simple string replace (first occurrence only)
            Value::String(s.replacen(&search, &replacement, 1))
        }
        "match" => {
            let regexp_str = match args.first() {
                Some(Value::NativeObject(props)) => {
                    // RegExp object - extract __regex__ property
                    props
                        .get("__regex__")
                        .map(|v| v.to_js_string())
                        .unwrap_or_default()
                }
                Some(v) => v.to_js_string(),
                None => String::new(),
            };
            let (pattern, flags) = parse_regexp_string(&regexp_str);

            if pattern.is_empty() && regexp_str.is_empty() {
                return Value::Null;
            }

            match simple_regex_match(&pattern, s, &flags) {
                Some((_start, matched)) => Value::String(matched),
                None => Value::Null,
            }
        }
        "search" => {
            let regexp_str = match args.first() {
                Some(Value::NativeObject(props)) => {
                    // RegExp object - extract __regex__ property
                    props
                        .get("__regex__")
                        .map(|v| v.to_js_string())
                        .unwrap_or_default()
                }
                Some(v) => v.to_js_string(),
                None => String::new(),
            };
            let (pattern, flags) = parse_regexp_string(&regexp_str);

            match simple_regex_match(&pattern, s, &flags) {
                Some((index, _)) => Value::Number(index as f64),
                None => Value::Number(-1.0),
            }
        }
        "concat" => {
            let mut result = s.to_string();
            for arg in args {
                result.push_str(&arg.to_js_string());
            }
            Value::String(result)
        }
        "toString" | "valueOf" => Value::String(s.to_string()),
        _ => Value::Undefined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_at() {
        let result = call_string_method("hello", "charAt", &[Value::Number(1.0)]);
        assert!(matches!(result, Value::String(s) if s == "e"));
    }

    #[test]
    fn test_index_of() {
        let result = call_string_method("hello world", "indexOf", &[Value::String("world".into())]);
        assert!(matches!(result, Value::Number(n) if n == 6.0));

        let result = call_string_method("hello world", "indexOf", &[Value::String("xyz".into())]);
        assert!(matches!(result, Value::Number(n) if n == -1.0));
    }

    #[test]
    fn test_to_lower_case() {
        let result = call_string_method("HELLO", "toLowerCase", &[]);
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_to_upper_case() {
        let result = call_string_method("hello", "toUpperCase", &[]);
        assert!(matches!(result, Value::String(s) if s == "HELLO"));
    }

    #[test]
    fn test_substring() {
        let result = call_string_method(
            "hello",
            "substring",
            &[Value::Number(1.0), Value::Number(3.0)],
        );
        assert!(matches!(result, Value::String(s) if s == "el"));
    }

    #[test]
    fn test_slice() {
        let result =
            call_string_method("hello", "slice", &[Value::Number(1.0), Value::Number(3.0)]);
        assert!(matches!(result, Value::String(s) if s == "el"));

        // Negative indices
        let result = call_string_method("hello", "slice", &[Value::Number(-2.0)]);
        assert!(matches!(result, Value::String(s) if s == "lo"));
    }

    #[test]
    fn test_trim() {
        let result = call_string_method("  hello  ", "trim", &[]);
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_replace() {
        let result = call_string_method(
            "hello world",
            "replace",
            &[Value::String("world".into()), Value::String("rust".into())],
        );
        assert!(matches!(result, Value::String(s) if s == "hello rust"));
    }

    #[test]
    fn test_concat() {
        let result = call_string_method(
            "hello",
            "concat",
            &[Value::String(" ".into()), Value::String("world".into())],
        );
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }
}
