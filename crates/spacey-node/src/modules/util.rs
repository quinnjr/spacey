// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `util` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the util module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Utility functions would be registered here
    exports.insert("types".to_string(), create_types_module());

    Value::NativeObject(exports)
}

/// Create util.types submodule
fn create_types_module() -> Value {
    let exports = HashMap::new();
    // Type checking functions would be registered here
    Value::NativeObject(exports)
}

/// util.format(format, ...args) - Printf-like formatting
pub fn format(fmt: &str, args: &[Value]) -> String {
    let mut result = String::new();
    let mut arg_index = 0;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    's' => {
                        chars.next();
                        if arg_index < args.len() {
                            result.push_str(&value_to_string(&args[arg_index]));
                            arg_index += 1;
                        } else {
                            result.push_str("%s");
                        }
                    }
                    'd' | 'i' => {
                        chars.next();
                        if arg_index < args.len() {
                            result.push_str(&value_to_number_string(&args[arg_index]));
                            arg_index += 1;
                        } else {
                            result.push_str(if next == 'd' { "%d" } else { "%i" });
                        }
                    }
                    'f' => {
                        chars.next();
                        if arg_index < args.len() {
                            result.push_str(&value_to_float_string(&args[arg_index]));
                            arg_index += 1;
                        } else {
                            result.push_str("%f");
                        }
                    }
                    'j' => {
                        chars.next();
                        if arg_index < args.len() {
                            result.push_str(&value_to_json(&args[arg_index]));
                            arg_index += 1;
                        } else {
                            result.push_str("%j");
                        }
                    }
                    'o' | 'O' => {
                        chars.next();
                        if arg_index < args.len() {
                            result.push_str(&inspect(&args[arg_index], None));
                            arg_index += 1;
                        } else {
                            result.push_str(if next == 'o' { "%o" } else { "%O" });
                        }
                    }
                    '%' => {
                        chars.next();
                        result.push('%');
                    }
                    _ => {
                        result.push('%');
                    }
                }
            } else {
                result.push('%');
            }
        } else {
            result.push(c);
        }
    }

    // Append remaining arguments
    for arg in args.iter().skip(arg_index) {
        result.push(' ');
        result.push_str(&value_to_string(arg));
    }

    result
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::BigInt(s) => format!("{}n", s),
        Value::Object(_) => "[object Object]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::NativeObject(_) => "[object Object]".to_string(),
    }
}

fn value_to_number_string(value: &Value) -> String {
    match value {
        Value::Number(n) => (*n as i64).to_string(),
        Value::String(s) => s
            .parse::<i64>()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "NaN".to_string()),
        Value::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
        _ => "NaN".to_string(),
    }
}

fn value_to_float_string(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s
            .parse::<f64>()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "NaN".to_string()),
        Value::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
        _ => "NaN".to_string(),
    }
}

fn value_to_json(value: &Value) -> String {
    // Simplified JSON serialization
    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"").replace('\n', "\\n")),
        Value::NativeObject(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", k, value_to_json(v)))
                .collect();
            format!("{{{}}}", items.join(","))
        }
        _ => "null".to_string(),
    }
}

/// util.inspect options
#[derive(Debug, Clone)]
pub struct InspectOptions {
    /// Show hidden properties
    pub show_hidden: bool,
    /// Depth to recurse
    pub depth: Option<u32>,
    /// Use colors
    pub colors: bool,
    /// Custom inspect function
    pub custom_inspect: bool,
    /// Show proxy
    pub show_proxy: bool,
    /// Max array length
    pub max_array_length: Option<usize>,
    /// Max string length
    pub max_string_length: Option<usize>,
    /// Break length
    pub break_length: usize,
    /// Compact
    pub compact: bool,
    /// Sorted
    pub sorted: bool,
    /// Getters
    pub getters: bool,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            depth: Some(2),
            colors: false,
            custom_inspect: true,
            show_proxy: false,
            max_array_length: Some(100),
            max_string_length: Some(10000),
            break_length: 80,
            compact: true,
            sorted: false,
            getters: false,
        }
    }
}

/// util.inspect - format a value for debugging
pub fn inspect(value: &Value, options: Option<InspectOptions>) -> String {
    let options = options.unwrap_or_default();
    inspect_value(value, &options, 0)
}

fn inspect_value(value: &Value, options: &InspectOptions, depth: u32) -> String {
    if let Some(max_depth) = options.depth {
        if depth > max_depth {
            return "[Object]".to_string();
        }
    }

    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_nan() {
                "NaN".to_string()
            } else if n.is_infinite() {
                if n.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => {
            let truncated = if let Some(max) = options.max_string_length {
                if s.len() > max {
                    format!("'{}... {} more characters'", &s[..max], s.len() - max)
                } else {
                    format!("'{}'", s)
                }
            } else {
                format!("'{}'", s)
            };
            truncated
        }
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::BigInt(s) => format!("{}n", s),
        Value::Object(_) => "[Object]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::NativeObject(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, inspect_value(v, options, depth + 1)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
    }
}

/// util.deprecate - mark a function as deprecated
pub fn deprecate(_fn: Value, _msg: &str, _code: Option<&str>) -> Value {
    // Would wrap function with deprecation warning
    _fn
}

/// util.inherits - prototype inheritance (legacy)
pub fn inherits(_constructor: &Value, _super_constructor: &Value) {
    // Legacy function for prototype inheritance
}

/// util.promisify - convert callback-style function to Promise
pub fn promisify(_original: Value) -> Value {
    // Would return a promisified version of the function
    Value::Undefined
}

/// util.callbackify - convert async function to callback-style
pub fn callbackify(_original: Value) -> Value {
    // Would return a callbackified version of the function
    Value::Undefined
}

/// util.isDeepStrictEqual - deep equality check
pub fn is_deep_strict_equal(val1: &Value, val2: &Value) -> bool {
    match (val1, val2) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            if a.is_nan() && b.is_nan() {
                true
            } else {
                a == b
            }
        }
        (Value::String(a), Value::String(b)) => a == b,
        // Object comparison (arrays are objects)
        (Value::Object(a), Value::Object(b)) => a == b,
        (Value::NativeObject(a), Value::NativeObject(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter().all(|(k, v)| {
                b.get(k)
                    .map(|bv| is_deep_strict_equal(v, bv))
                    .unwrap_or(false)
            })
        }
        _ => false,
    }
}

/// util.types - type checking utilities
pub mod types {
    use spacey_spidermonkey::Value;

    pub fn is_array_buffer(_value: &Value) -> bool {
        false // Would check for ArrayBuffer
    }

    pub fn is_array_buffer_view(_value: &Value) -> bool {
        false
    }

    pub fn is_async_function(_value: &Value) -> bool {
        false
    }

    pub fn is_big_int64_array(_value: &Value) -> bool {
        false
    }

    pub fn is_boolean_object(value: &Value) -> bool {
        matches!(value, Value::Boolean(_))
    }

    pub fn is_date(_value: &Value) -> bool {
        false // Would check for Date object
    }

    pub fn is_float32_array(_value: &Value) -> bool {
        false
    }

    pub fn is_float64_array(_value: &Value) -> bool {
        false
    }

    pub fn is_generator_function(_value: &Value) -> bool {
        false
    }

    pub fn is_generator_object(_value: &Value) -> bool {
        false
    }

    pub fn is_int8_array(_value: &Value) -> bool {
        false
    }

    pub fn is_int16_array(_value: &Value) -> bool {
        false
    }

    pub fn is_int32_array(_value: &Value) -> bool {
        false
    }

    pub fn is_map(_value: &Value) -> bool {
        false
    }

    pub fn is_map_iterator(_value: &Value) -> bool {
        false
    }

    pub fn is_native_error(_value: &Value) -> bool {
        false
    }

    pub fn is_number_object(value: &Value) -> bool {
        matches!(value, Value::Number(_))
    }

    pub fn is_promise(_value: &Value) -> bool {
        false
    }

    pub fn is_proxy(_value: &Value) -> bool {
        false
    }

    pub fn is_reg_exp(_value: &Value) -> bool {
        false
    }

    pub fn is_set(_value: &Value) -> bool {
        false
    }

    pub fn is_set_iterator(_value: &Value) -> bool {
        false
    }

    pub fn is_string_object(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    pub fn is_symbol_object(_value: &Value) -> bool {
        false
    }

    pub fn is_typed_array(_value: &Value) -> bool {
        false
    }

    pub fn is_uint8_array(_value: &Value) -> bool {
        false
    }

    pub fn is_uint8_clamped_array(_value: &Value) -> bool {
        false
    }

    pub fn is_uint16_array(_value: &Value) -> bool {
        false
    }

    pub fn is_uint32_array(_value: &Value) -> bool {
        false
    }

    pub fn is_weak_map(_value: &Value) -> bool {
        false
    }

    pub fn is_weak_set(_value: &Value) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_string() {
        let result = format("Hello, %s!", &[Value::String("World".to_string())]);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_format_number() {
        let result = format("Count: %d", &[Value::Number(42.0)]);
        assert_eq!(result, "Count: 42");
    }

    #[test]
    fn test_format_multiple() {
        let result = format(
            "%s is %d years old",
            &[Value::String("Alice".to_string()), Value::Number(30.0)],
        );
        assert_eq!(result, "Alice is 30 years old");
    }

    #[test]
    fn test_format_extra_args() {
        let result = format(
            "Hello",
            &[Value::String("World".to_string()), Value::Number(42.0)],
        );
        assert_eq!(result, "Hello World 42");
    }

    #[test]
    fn test_inspect_string() {
        let result = inspect(&Value::String("hello".to_string()), None);
        assert_eq!(result, "'hello'");
    }

    #[test]
    fn test_inspect_array() {
        // Create array-like object (arrays are represented as NativeObject in spacey)
        fn make_array(values: Vec<Value>) -> Value {
            let mut obj = std::collections::HashMap::new();
            for (i, v) in values.into_iter().enumerate() {
                obj.insert(i.to_string(), v);
            }
            obj.insert("length".to_string(), Value::Number(obj.len() as f64));
            Value::NativeObject(obj)
        }

        let arr = make_array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = inspect(&arr, None);
        // NativeObject inspection won't give exact array format, but should work
        assert!(result.contains("1") && result.contains("2") && result.contains("3"));
    }

    #[test]
    fn test_is_deep_strict_equal() {
        assert!(is_deep_strict_equal(
            &Value::Number(1.0),
            &Value::Number(1.0)
        ));
        assert!(!is_deep_strict_equal(
            &Value::Number(1.0),
            &Value::Number(2.0)
        ));

        // Create array-like objects
        fn make_array(values: Vec<Value>) -> Value {
            let mut obj = std::collections::HashMap::new();
            for (i, v) in values.into_iter().enumerate() {
                obj.insert(i.to_string(), v);
            }
            obj.insert("length".to_string(), Value::Number(obj.len() as f64));
            Value::NativeObject(obj)
        }

        let arr1 = make_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let arr2 = make_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert!(is_deep_strict_equal(&arr1, &arr2));
    }
}
