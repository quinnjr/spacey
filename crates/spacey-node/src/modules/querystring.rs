// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `querystring` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the querystring module exports
pub fn create_module() -> Value {
    let exports = HashMap::new();
    Value::NativeObject(exports)
}

/// Parse a query string into an object
pub fn parse(qs: &str, sep: Option<&str>, eq: Option<&str>) -> HashMap<String, Vec<String>> {
    let sep = sep.unwrap_or("&");
    let eq = eq.unwrap_or("=");

    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for part in qs.split(sep) {
        if part.is_empty() {
            continue;
        }

        let mut iter = part.splitn(2, eq);
        let key = decode(iter.next().unwrap_or(""));
        let value = decode(iter.next().unwrap_or(""));

        result.entry(key).or_default().push(value);
    }

    result
}

/// Stringify an object into a query string
pub fn stringify(
    obj: &HashMap<String, Vec<String>>,
    sep: Option<&str>,
    eq: Option<&str>,
) -> String {
    let sep = sep.unwrap_or("&");
    let eq = eq.unwrap_or("=");

    let mut parts = Vec::new();

    for (key, values) in obj {
        let encoded_key = encode(key);
        for value in values {
            let encoded_value = encode(value);
            parts.push(format!("{}{}{}", encoded_key, eq, encoded_value));
        }
    }

    parts.join(sep)
}

/// URL-encode a string
pub fn encode(s: &str) -> String {
    let mut result = String::new();

    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }

    result
}

/// Alias for encode
pub fn escape(s: &str) -> String {
    encode(s)
}

/// URL-decode a string
pub fn decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                result.push(byte);
                i += 3;
                continue;
            }
        }

        if bytes[i] == b'+' {
            result.push(b' ');
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Alias for decode
pub fn unescape(s: &str) -> String {
    decode(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("foo=bar&baz=qux", None, None);
        assert_eq!(result.get("foo"), Some(&vec!["bar".to_string()]));
        assert_eq!(result.get("baz"), Some(&vec!["qux".to_string()]));
    }

    #[test]
    fn test_parse_multiple_values() {
        let result = parse("foo=bar&foo=baz", None, None);
        assert_eq!(
            result.get("foo"),
            Some(&vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn test_stringify() {
        let mut obj = HashMap::new();
        obj.insert("foo".to_string(), vec!["bar".to_string()]);
        obj.insert("baz".to_string(), vec!["qux".to_string()]);

        let result = stringify(&obj, None, None);
        assert!(result.contains("foo=bar"));
        assert!(result.contains("baz=qux"));
    }

    #[test]
    fn test_encode_decode() {
        let original = "hello world!";
        let encoded = encode(original);
        let decoded = decode(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_special() {
        let encoded = encode("hello world");
        assert_eq!(encoded, "hello%20world");
    }
}
