//! The bytecode interpreter.

use std::collections::HashMap;
use std::sync::Arc;

use crate::Error;
use crate::compiler::{Bytecode, OpCode, Operand};
use crate::runtime::function::{CallFrame, Callable, Function};
use crate::runtime::value::Value;

/// Abstract equality comparison (ES3 Section 11.9.3)
///
/// The Abstract Equality Comparison Algorithm with type coercion.
fn abstract_equals(a: &Value, b: &Value) -> bool {
    // 1. If Type(x) is the same as Type(y), return strict equality
    match (a, b) {
        // Same type comparisons
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // NaN != NaN in JavaScript
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a == b
            }
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Object(a), Value::Object(b)) => a == b,
        (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b),

        // 2. null == undefined is true
        (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,

        // 3. If one is number and other is string, convert string to number
        (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
            let s_num = s.parse::<f64>().unwrap_or(f64::NAN);
            if s_num.is_nan() || n.is_nan() {
                false
            } else {
                n == &s_num
            }
        }

        // 4. If one is boolean, convert it to number and compare
        (Value::Boolean(b_val), other) => {
            let num = if *b_val { 1.0 } else { 0.0 };
            abstract_equals(&Value::Number(num), other)
        }
        (other, Value::Boolean(b_val)) => {
            let num = if *b_val { 1.0 } else { 0.0 };
            abstract_equals(other, &Value::Number(num))
        }

        // 5. If one is number/string and other is object, convert object to primitive
        // For now, simplified handling
        (Value::Number(_) | Value::String(_), Value::Object(_)) => {
            // ToPrimitive(object) - simplified, compare as false
            false
        }
        (Value::Object(_), Value::Number(_) | Value::String(_)) => {
            // ToPrimitive(object) - simplified, compare as false
            false
        }

        // All other cases are not equal
        _ => false,
    }
}

/// A saved call frame for restoring after return.
#[derive(Clone)]
#[allow(dead_code)]
struct SavedFrame {
    /// Saved instruction pointer
    ip: usize,
    /// Saved bytecode reference
    bytecode_idx: usize,
    /// Saved locals base
    locals_base: usize,
}

/// Call a Date method
fn call_date_method(timestamp: f64, method: &str, _args: &[Value]) -> Value {
    if timestamp.is_nan() || timestamp.is_infinite() {
        return Value::Number(f64::NAN);
    }

    // Helper to extract date components from timestamp
    // Timestamp is milliseconds since Unix epoch (Jan 1, 1970)
    let ms = timestamp as i64;
    let secs = ms / 1000;
    let millis = (ms % 1000) as f64;

    // Days since epoch
    let days_since_epoch = secs / 86400;

    // Calculate year, month, day
    let (year, month, day, day_of_week) = days_to_ymd(days_since_epoch);

    // Calculate hours, minutes, seconds
    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    match method {
        "getTime" | "valueOf" => Value::Number(timestamp),
        "getFullYear" => Value::Number(year as f64),
        "getMonth" => Value::Number(month as f64), // 0-indexed
        "getDate" => Value::Number(day as f64),
        "getDay" => Value::Number(day_of_week as f64),
        "getHours" => Value::Number(hours as f64),
        "getMinutes" => Value::Number(minutes as f64),
        "getSeconds" => Value::Number(seconds as f64),
        "getMilliseconds" => Value::Number(millis),
        "toString" => Value::String(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year, month + 1, day, hours, minutes, seconds, millis as i64
        )),
        "toDateString" => Value::String(format!("{:04}-{:02}-{:02}", year, month + 1, day)),
        "toTimeString" => Value::String(format!("{:02}:{:02}:{:02}", hours, minutes, seconds)),
        _ => Value::Undefined,
    }
}

/// Convert days since Unix epoch to (year, month, day, day_of_week)
fn days_to_ymd(days: i64) -> (i32, i32, i32, i32) {
    // Simplified date calculation
    // Note: This is a basic implementation, may have edge cases
    let remaining_days = days + 719468; // Days from year 0 to 1970

    // Calculate year
    let era = if remaining_days >= 0 { remaining_days } else { remaining_days - 146096 } / 146097;
    let doe = (remaining_days - era * 146097) as i32; // Day of era
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // Year of era
    let year = yoe + (era as i32) * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // Day of year
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    // Day of week (0 = Sunday)
    let day_of_week = ((days + 4) % 7 + 7) % 7;

    (year, month - 1, day, day_of_week as i32) // month is 0-indexed
}

/// Call a number method
fn call_number_method(n: f64, method: &str, args: &[Value]) -> Value {
    match method {
        "toString" => {
            let radix = args.first().map(|v| v.to_number() as u32).unwrap_or(10);
            if radix == 10 {
                Value::String(n.to_string())
            } else if (2..=36).contains(&radix) {
                let int_val = n as i64;
                Value::String(format_radix(int_val, radix))
            } else {
                Value::String(n.to_string())
            }
        }
        "toFixed" => {
            let digits = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Value::String(format!("{:.prec$}", n, prec = digits))
        }
        "toExponential" => {
            let digits = args.first().map(|v| v.to_number() as usize);
            match digits {
                Some(d) => Value::String(format!("{:.prec$e}", n, prec = d)),
                None => Value::String(format!("{:e}", n)),
            }
        }
        "toPrecision" => {
            let precision = args.first().map(|v| v.to_number() as usize).unwrap_or(6);
            Value::String(format!("{:.prec$}", n, prec = precision.saturating_sub(1)))
        }
        "valueOf" => Value::Number(n),
        _ => Value::Undefined,
    }
}

/// Format an integer in a given radix
fn format_radix(mut n: i64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let negative = n < 0;
    if negative {
        n = -n;
    }
    let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    while n > 0 {
        let digit = (n % radix as i64) as usize;
        result.push(digits.chars().nth(digit).unwrap());
        n /= radix as i64;
    }
    if negative {
        result.push('-');
    }
    result.chars().rev().collect()
}

/// Call a string method
fn call_string_method(s: &str, method: &str, args: &[Value]) -> Value {
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
            let end = args.get(1).map(|v| v.to_number() as i32).unwrap_or(s.len() as i32);
            let len = s.len() as i32;
            let start = start.max(0).min(len) as usize;
            let end = end.max(0).min(len) as usize;
            let (start, end) = if start > end { (end, start) } else { (start, end) };
            Value::String(s.chars().skip(start).take(end - start).collect())
        }
        "slice" => {
            let len = s.len() as i32;
            let start = args.first().map(|v| v.to_number() as i32).unwrap_or(0);
            let end = args.get(1).map(|v| v.to_number() as i32).unwrap_or(len);
            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
            let end = if end < 0 { (len + end).max(0) } else { end.min(len) } as usize;
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
            let start = if start < 0 { (s_len + start).max(0) } else { start } as usize;
            let length = len_arg.unwrap_or(s_len - start as i32).max(0) as usize;
            Value::String(s.chars().skip(start).take(length).collect())
        }
        "toLowerCase" => {
            Value::String(s.to_lowercase())
        }
        "toUpperCase" => {
            Value::String(s.to_uppercase())
        }
        "split" => {
            let separator = args.first().map(|v| v.to_js_string()).unwrap_or_default();
            let parts: Vec<Value> = if separator.is_empty() {
                s.chars().map(|c| Value::String(c.to_string())).collect()
            } else {
                s.split(&separator).map(|p| Value::String(p.to_string())).collect()
            };
            // Return as a simple object representing array (VM will handle creation)
            // For now, return a marker that the VM can process
            Value::String(format!("__split_result__{}:{}", parts.len(), parts.iter().map(|v| v.to_js_string()).collect::<Vec<_>>().join("\x00")))
        }
        "trim" => {
            Value::String(s.trim().to_string())
        }
        "replace" => {
            let (search, is_regexp) = match args.first() {
                Some(Value::NativeObject(props)) => {
                    // RegExp object - extract __regex__ property
                    let regex_str = props.get("__regex__")
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
                    props.get("__regex__")
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
                    props.get("__regex__")
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
        "toString" | "valueOf" => {
            Value::String(s.to_string())
        }
        _ => Value::Undefined,
    }
}

/// Call a regexp method
fn call_regexp_method(regex_str: &str, method: &str, args: &[Value]) -> Value {
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
        format!("{}{}{}", before, replacement, after)
    } else {
        input.to_string()
    }
}

/// Replace all matches in a string
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
        result.push_str(replacement);
        start = abs_idx + pattern.len().max(1);
    }

    result.push_str(&input[start..]);
    result
}

/// Runtime object representation
#[derive(Clone, Debug)]
pub struct RuntimeObject {
    /// Properties stored as key-value pairs
    properties: HashMap<String, Value>,
    /// For arrays: stores elements by index
    array_elements: Vec<Value>,
    /// Whether this is an array
    is_array: bool,
}

impl RuntimeObject {
    /// Create a new empty object
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
            array_elements: Vec::new(),
            is_array: false,
        }
    }

    /// Create a new array with elements
    fn new_array(elements: Vec<Value>) -> Self {
        let len = elements.len();
        let mut obj = Self {
            properties: HashMap::new(),
            array_elements: elements,
            is_array: true,
        };
        obj.properties.insert("length".to_string(), Value::Number(len as f64));
        obj
    }

    /// Get a property by name
    fn get(&self, name: &str) -> Value {
        if self.is_array {
            // Check for numeric index
            if let Ok(idx) = name.parse::<usize>() {
                return self.array_elements.get(idx).cloned().unwrap_or(Value::Undefined);
            }
        }
        self.properties.get(name).cloned().unwrap_or(Value::Undefined)
    }

    /// Check if this is an array
    fn is_array(&self) -> bool {
        self.is_array
    }

    /// Get array elements (for array methods)
    fn elements(&self) -> &[Value] {
        &self.array_elements
    }

    /// Push an element to the array, returns new length
    fn array_push(&mut self, value: Value) -> f64 {
        if self.is_array {
            self.array_elements.push(value);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            len
        } else {
            0.0
        }
    }

    /// Pop an element from the array
    fn array_pop(&mut self) -> Value {
        if self.is_array {
            let result = self.array_elements.pop().unwrap_or(Value::Undefined);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            result
        } else {
            Value::Undefined
        }
    }

    /// Shift (remove first element)
    fn array_shift(&mut self) -> Value {
        if self.is_array && !self.array_elements.is_empty() {
            let result = self.array_elements.remove(0);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            result
        } else {
            Value::Undefined
        }
    }

    /// Unshift (add to beginning), returns new length
    fn array_unshift(&mut self, value: Value) -> f64 {
        if self.is_array {
            self.array_elements.insert(0, value);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            len
        } else {
            0.0
        }
    }

    /// Join elements with separator
    fn array_join(&self, separator: &str) -> String {
        if self.is_array {
            self.array_elements
                .iter()
                .map(|v| v.to_js_string())
                .collect::<Vec<_>>()
                .join(separator)
        } else {
            String::new()
        }
    }

    /// Reverse the array in place
    fn array_reverse(&mut self) {
        if self.is_array {
            self.array_elements.reverse();
        }
    }

    /// Slice the array
    fn array_slice(&self, start: i32, end: i32) -> Vec<Value> {
        if self.is_array {
            let len = self.array_elements.len() as i32;
            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
            let end = if end < 0 { (len + end).max(0) } else { end.min(len) } as usize;
            if start >= end {
                vec![]
            } else {
                self.array_elements[start..end].to_vec()
            }
        } else {
            vec![]
        }
    }

    /// Index of an element
    fn array_index_of(&self, value: &Value) -> i32 {
        if self.is_array {
            for (i, elem) in self.array_elements.iter().enumerate() {
                if elem == value {
                    return i as i32;
                }
            }
        }
        -1
    }

    /// Concat arrays
    fn array_concat(&self, other: &[Value]) -> Vec<Value> {
        if self.is_array {
            let mut result = self.array_elements.clone();
            result.extend(other.iter().cloned());
            result
        } else {
            other.to_vec()
        }
    }

    /// Set a property
    fn set(&mut self, name: &str, value: Value) {
        if self.is_array
            && let Ok(idx) = name.parse::<usize>() {
                // Extend array if necessary
                while self.array_elements.len() <= idx {
                    self.array_elements.push(Value::Undefined);
                }
                self.array_elements[idx] = value;
                // Update length
                self.properties.insert("length".to_string(), Value::Number(self.array_elements.len() as f64));
                return;
            }
        self.properties.insert(name.to_string(), value);
    }

    /// Get all enumerable property keys
    fn keys(&self) -> Vec<String> {
        if self.is_array {
            // For arrays, return indices as strings plus property keys
            let mut keys: Vec<String> = (0..self.array_elements.len())
                .map(|i| i.to_string())
                .collect();
            // Add non-index properties (except length)
            for key in self.properties.keys() {
                if key != "length" && key.parse::<usize>().is_err() {
                    keys.push(key.clone());
                }
            }
            keys
        } else {
            // For regular objects, return all property keys
            self.properties.keys().cloned().collect()
        }
    }

    /// Delete a property
    fn delete(&mut self, name: &str) -> bool {
        if self.is_array
            && let Ok(idx) = name.parse::<usize>()
                && idx < self.array_elements.len() {
                    self.array_elements[idx] = Value::Undefined;
                    return true;
                }
        self.properties.remove(name).is_some()
    }
}

/// The virtual machine that executes bytecode.
#[derive(Clone)]
pub struct VM {
    /// The value stack
    stack: Vec<Value>,
    /// Local variables
    locals: Vec<Value>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Instruction pointer
    ip: usize,
    /// Call stack for function calls
    #[allow(dead_code)]
    call_stack: Vec<SavedFrame>,
    /// Native functions
    native_functions: HashMap<String, Arc<Callable>>,
    /// Heap for runtime objects
    heap: Vec<RuntimeObject>,
}

impl VM {
    /// Creates a new VM.
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::with_capacity(256),
            locals: Vec::with_capacity(64),
            globals: HashMap::new(),
            ip: 0,
            call_stack: Vec::with_capacity(64),
            native_functions: HashMap::new(),
            heap: Vec::with_capacity(256),
        };
        vm.register_builtins();
        vm
    }

    /// Allocate a new object on the heap, returning its index
    fn alloc_object(&mut self, obj: RuntimeObject) -> usize {
        let idx = self.heap.len();
        self.heap.push(obj);
        idx
    }

    /// Register built-in native functions.
    fn register_builtins(&mut self) {
        // Register all builtins from the builtins module
        let builtins = crate::builtins::register_builtins();
        for (name, value) in builtins {
            self.globals.insert(name, value);
        }
    }

    /// Register a native function.
    pub fn register_native(
        &mut self,
        name: &str,
        arity: i32,
        func: fn(&mut CallFrame, &[Value]) -> Result<Value, String>,
    ) {
        let callable = Callable::Native {
            name: name.to_string(),
            arity,
            func,
        };
        self.native_functions
            .insert(name.to_string(), Arc::new(callable));
    }

    /// Get a native function by name.
    pub fn get_native(&self, name: &str) -> Option<Arc<Callable>> {
        self.native_functions.get(name).cloned()
    }

    /// Call a user-defined function with given arguments.
    #[allow(dead_code)]
    fn call_function(&mut self, func: &Function, args: &[Value]) -> Result<Value, Error> {
        self.call_function_with_value(func, args, None)
    }

    /// Call a user-defined function with given arguments and optional function value for named expressions.
    fn call_function_with_value(
        &mut self,
        func: &Function,
        args: &[Value],
        func_value: Option<Value>,
    ) -> Result<Value, Error> {
        // Execute the function's bytecode
        let func_bytecode = &func.bytecode;

        // Save current state
        let saved_ip = self.ip;
        let saved_locals_len = self.locals.len();

        // For named function expressions, the function name is local[0]
        // and parameters start at local[1]
        let param_offset = if func.name.is_some() { 1 } else { 0 };

        // Allocate locals
        let total_locals = func.local_count.max(func.params.len() + param_offset);
        while self.locals.len() < saved_locals_len + total_locals {
            self.locals.push(Value::Undefined);
        }

        // For named function expressions, set local[0] to the function itself
        if func.name.is_some()
            && let Some(ref fv) = func_value {
                self.locals[saved_locals_len] = fv.clone();
            }

        // Set up parameters
        for (i, _param) in func.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(Value::Undefined);
            self.locals[saved_locals_len + param_offset + i] = value;
        }

        // Create the arguments object (ES3 Section 10.1.8)
        let arguments_obj_idx = self.alloc_object(RuntimeObject::new_array(args.to_vec()));
        // Also set callee property (the function itself)
        if let Some(ref fv) = func_value
            && let Some(obj) = self.heap.get_mut(arguments_obj_idx) {
                obj.set("callee", fv.clone());
            }
        // Save previous arguments if any, and set new arguments
        let prev_arguments = self.globals.remove("arguments");
        self.globals
            .insert("arguments".to_string(), Value::Object(arguments_obj_idx));

        // Apply closure environment - only inject values if they don't already exist in globals
        // This allows closure state to persist across multiple calls
        let saved_closure_values: std::collections::HashMap<String, Option<Value>> =
            std::collections::HashMap::new();
        for (name, value) in &func.closure_env {
            // Only inject if the variable doesn't already exist in globals
            // (existing value means it was set by a previous call to this closure)
            if !self.globals.contains_key(name) {
                self.globals.insert(name.clone(), value.clone());
            }
        }

        // Execute function
        self.ip = 0;
        let mut result = Value::Undefined;

        loop {
            if self.ip >= func_bytecode.instructions.len() {
                break;
            }

            let instruction = &func_bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Return => {
                    result = self.pop().unwrap_or(Value::Undefined);
                    break;
                }

                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = func_bytecode.constants[*idx as usize].clone();
                        // If loading a function, capture the current closure environment
                        let value = if let Value::Function(callable) = &value {
                            if let Callable::Function(inner_func) = callable.as_ref() {
                                // Clone the function with the current globals as closure env
                                // This captures variables from the enclosing scope
                                let closure_env = self.globals.clone();
                                let mut new_func = inner_func.clone();
                                new_func.closure_env = closure_env;
                                Value::Function(Arc::new(Callable::Function(new_func)))
                            } else {
                                value
                            }
                        } else {
                            value
                        };
                        self.stack.push(value);
                    }
                }

                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let local_idx = saved_locals_len + *idx as usize;
                        let value = self
                            .locals
                            .get(local_idx)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let local_idx = saved_locals_len + *idx as usize;
                        let value = self.pop()?;
                        while self.locals.len() <= local_idx {
                            self.locals.push(Value::Undefined);
                        }
                        self.locals[local_idx] = value;
                    }
                }

                OpCode::LoadUndefined => {
                    self.stack.push(Value::Undefined);
                }

                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,

                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::LoadGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        let name = match &func_bytecode.constants[*idx as usize] {
                            Value::String(s) => s.clone(),
                            _ => return Err(Error::TypeError("Property name must be a string".into())),
                        };
                        let value = self.globals.get(&name).cloned().unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        let name = match &func_bytecode.constants[*idx as usize] {
                            Value::String(s) => s.clone(),
                            _ => return Err(Error::TypeError("Property name must be a string".into())),
                        };
                        let value = self.pop()?;
                        self.globals.insert(name, value);
                    }
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                OpCode::Swap => {
                    let len = self.stack.len();
                    if len >= 2 {
                        self.stack.swap(len - 1, len - 2);
                    }
                }

                OpCode::LoadTrue => {
                    self.stack.push(Value::Boolean(true));
                }

                OpCode::LoadFalse => {
                    self.stack.push(Value::Boolean(false));
                }

                OpCode::LoadNull => {
                    self.stack.push(Value::Null);
                }

                // Comparison operations
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Le => self.compare_op(|a, b| a <= b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Ge => self.compare_op(|a, b| a >= b)?,
                OpCode::Eq => {
                    // Abstract equality (ES3 Section 11.9.3)
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(abstract_equals(&a, &b)));
                }
                OpCode::StrictEq => {
                    // Strict equality (ES3 Section 11.9.6)
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Ne => {
                    // Abstract inequality
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(!abstract_equals(&a, &b)));
                }
                OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a != b));
                }

                // Jump operations
                OpCode::Jump => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        self.ip = *target as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if !condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                OpCode::JumpIfTrue => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                OpCode::Neg => {
                    if let Some(Value::Number(n)) = self.stack.pop() {
                        self.stack.push(Value::Number(-n));
                    }
                }

                OpCode::Not => {
                    let value = self.pop()?;
                    self.stack.push(Value::Boolean(!value.to_boolean()));
                }

                // Property access
                OpCode::GetProperty => {
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        match func_bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    let obj = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = match &obj {
                        Value::Number(n) => {
                            // Number properties
                            match prop_name.as_str() {
                                "toString" | "toFixed" | "toExponential" | "toPrecision" | "valueOf" => {
                                    Value::String(format!("__number_method__{}:{}", prop_name, n))
                                }
                                _ => Value::Undefined,
                            }
                        }
                        Value::String(s) => match prop_name.as_str() {
                            "length" => Value::Number(s.len() as f64),
                            // String prototype methods
                            "charAt" | "charCodeAt" | "indexOf" | "lastIndexOf" |
                            "substring" | "slice" | "substr" | "toLowerCase" | "toUpperCase" |
                            "split" | "trim" | "replace" | "concat" | "toString" | "valueOf" |
                            "match" | "search" => {
                                // Store string value in a temporary location for method call
                                // Use a marker that includes the string value encoded
                                Value::String(format!("__string_method__{}:{}", prop_name, s))
                            }
                            _ => {
                                if let Ok(idx) = prop_name.parse::<usize>() {
                                    s.chars()
                                        .nth(idx)
                                        .map(|c| Value::String(c.to_string()))
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::Undefined
                                }
                            }
                        },
                        Value::Object(heap_idx) => {
                            if let Some(heap_obj) = self.heap.get(*heap_idx) {
                                // Check for array methods
                                if heap_obj.is_array() {
                                    match prop_name.as_str() {
                                        "push" | "pop" | "shift" | "unshift" | "splice" | "slice" |
                                        "concat" | "join" | "reverse" | "sort" | "indexOf" | "lastIndexOf" |
                                        "toString" | "toLocaleString" => {
                                            // Return a bound method marker
                                            Value::String(format!("__array_method__{}_{}", prop_name, heap_idx))
                                        }
                                        _ => heap_obj.get(&prop_name)
                                    }
                                } else {
                                    // Check if this is a Date object
                                    if let Value::String(type_str) = heap_obj.get("__type__") {
                                        if type_str == "Date" {
                                            match prop_name.as_str() {
                                                "getTime" | "getFullYear" | "getMonth" | "getDate" |
                                                "getDay" | "getHours" | "getMinutes" | "getSeconds" |
                                                "getMilliseconds" | "toString" | "toDateString" |
                                                "toTimeString" | "valueOf" => {
                                                    let timestamp = if let Value::Number(ts) = heap_obj.get("__timestamp__") {
                                                        ts
                                                    } else {
                                                        f64::NAN
                                                    };
                                                    Value::String(format!("__date_method__{}:{}", prop_name, timestamp))
                                                }
                                                _ => heap_obj.get(&prop_name)
                                            }
                                        } else {
                                            heap_obj.get(&prop_name)
                                        }
                                    } else {
                                        heap_obj.get(&prop_name)
                                    }
                                }
                            } else {
                                Value::Undefined
                            }
                        }
                        Value::NativeObject(props) => {
                            props.get(&prop_name).cloned().unwrap_or(Value::Undefined)
                        }
                        _ => Value::Undefined,
                    };
                    self.stack.push(result);
                }

                // Function calls within functions
                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;
                        let mut call_args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            call_args.push(self.pop()?);
                        }
                        call_args.reverse();

                        let callee = self.pop()?;
                        let callee_for_named = callee.clone(); // Clone for named function expressions

                        // Check for array method marker (__array_method__METHOD_HEAPIDX)
                        if let Value::String(s) = &callee {
                            if let Some(rest) = s.strip_prefix("__array_method__")
                                && let Some(last_underscore) = rest.rfind('_') {
                                    let method = &rest[..last_underscore];
                                    let heap_idx: usize = rest[last_underscore + 1..].parse().unwrap_or(0);
                                    let result = self.call_array_method(heap_idx, method, &call_args)?;
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for string method marker (__string_method__METHOD:STRING)
                            if let Some(rest) = s.strip_prefix("__string_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let string_val = &rest[colon_pos + 1..];
                                    // Handle split specially to create an actual array
                                    if method == "split" {
                                        let separator = call_args.first().map(|v| v.to_js_string()).unwrap_or_default();
                                        let parts: Vec<Value> = if separator.is_empty() {
                                            string_val.chars().map(|c| Value::String(c.to_string())).collect()
                                        } else {
                                            string_val.split(&separator).map(|p| Value::String(p.to_string())).collect()
                                        };
                                        let arr_idx = self.alloc_object(RuntimeObject::new_array(parts));
                                        self.stack.push(Value::Object(arr_idx));
                                    } else {
                                        let result = call_string_method(string_val, method, &call_args);
                                        self.stack.push(result);
                                    }
                                    continue;
                                }
                            // Check for number method marker (__number_method__METHOD:NUMBER)
                            if let Some(rest) = s.strip_prefix("__number_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let num_val: f64 = rest[colon_pos + 1..].parse().unwrap_or(f64::NAN);
                                    let result = call_number_method(num_val, method, &call_args);
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for date method marker (__date_method__METHOD:TIMESTAMP)
                            if let Some(rest) = s.strip_prefix("__date_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let timestamp: f64 = rest[colon_pos + 1..].parse().unwrap_or(f64::NAN);
                                    let result = call_date_method(timestamp, method, &call_args);
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for regexp method marker (__regexp_method__METHOD:/pattern/flags)
                            if let Some(rest) = s.strip_prefix("__regexp_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let regex_str = &rest[colon_pos + 1..];
                                    let result = call_regexp_method(regex_str, method, &call_args);
                                    self.stack.push(result);
                                    continue;
                                }
                        }

                        match callee {
                            Value::Function(callable) => {
                                match callable.as_ref() {
                                    crate::runtime::function::Callable::Native { func, .. } => {
                                        let temp_func = Function::new(None, vec![], Bytecode::new(), 0);
                                        let mut frame = CallFrame::new(temp_func, 0);
                                        match func(&mut frame, &call_args) {
                                            Ok(res) => self.stack.push(res),
                                            Err(e) => return Err(Error::TypeError(e)),
                                        }
                                    }
                                    crate::runtime::function::Callable::Function(inner_func) => {
                                        // Pass the function value for named function expressions
                                        let res = self.call_function_with_value(
                                            inner_func,
                                            &call_args,
                                            Some(callee_for_named),
                                        )?;
                                        self.stack.push(res);
                                    }
                                }
                            }
                            _ => return Err(Error::TypeError("Value is not callable".into())),
                        }
                    }
                }

                OpCode::MakeClosure => {
                    // Pop variable names string from stack
                    let var_names_value = self.pop()?;
                    let var_names_str = match var_names_value {
                        Value::String(s) => s,
                        _ => String::new(),
                    };

                    // Pop the base function from stack
                    let base_func = self.pop()?;

                    // Create closure environment by capturing variables
                    // Look up from both globals AND locals of the current function
                    let mut closure_env = std::collections::HashMap::new();
                    if !var_names_str.is_empty() {
                        for var_name in var_names_str.split(',') {
                            let var_name = var_name.trim();
                            if !var_name.is_empty() {
                                // First try globals
                                if let Some(value) = self.globals.get(var_name) {
                                    closure_env.insert(var_name.to_string(), value.clone());
                                } else {
                                    // Try to find in current function's locals by looking up the param/local name
                                    // For params: func.params[i] is at saved_locals_len + param_offset + i
                                    let mut found = false;

                                    // Check parameters
                                    for (i, param) in func.params.iter().enumerate() {
                                        if param == var_name {
                                            let local_idx = saved_locals_len + param_offset + i;
                                            if let Some(value) = self.locals.get(local_idx) {
                                                closure_env.insert(var_name.to_string(), value.clone());
                                                found = true;
                                                break;
                                            }
                                        }
                                    }

                                    // If not found in params, check if it's a local variable
                                    // For this we need to iterate through all locals after params
                                    // But we don't have the variable names at runtime...
                                    // As a fallback, store undefined
                                    if !found {
                                        // Try all remaining locals (after params) as a heuristic
                                        // This won't work perfectly but handles simple cases
                                        closure_env.insert(var_name.to_string(), Value::Undefined);
                                    }
                                }
                            }
                        }
                    }

                    // Create new function with closure environment
                    if let Value::Function(callable) = base_func {
                        match callable.as_ref() {
                            Callable::Function(inner_func) => {
                                let new_func = Function::new_with_closure(
                                    inner_func.name.clone(),
                                    inner_func.params.clone(),
                                    inner_func.bytecode.clone(),
                                    inner_func.local_count,
                                    closure_env,
                                );
                                let new_callable = Callable::Function(new_func);
                                self.stack.push(Value::Function(Arc::new(new_callable)));
                            }
                            _ => {
                                // Native functions don't support closures, just push as-is
                                self.stack.push(Value::Function(callable));
                            }
                        }
                    } else {
                        self.stack.push(base_func);
                    }
                }

                _ => {
                    // For any other operations, we'd need to handle them
                    // For now, skip unhandled ops in function bodies
                }
            }
        }

        // Restore state
        self.ip = saved_ip;
        self.locals.truncate(saved_locals_len);

        // Note: We don't restore closure environment values because closures need
        // to persist their modified state across calls. The closure variables
        // remain in globals so subsequent calls can see the updated values.
        // This is intentional for proper closure semantics.
        let _ = saved_closure_values; // Suppress unused warning

        // Restore previous arguments object
        if let Some(prev) = prev_arguments {
            self.globals.insert("arguments".to_string(), prev);
        } else {
            self.globals.remove("arguments");
        }

        Ok(result)
    }

    /// Call an array method on a heap object
    fn call_array_method(&mut self, heap_idx: usize, method: &str, args: &[Value]) -> Result<Value, Error> {
        match method {
            "push" => {
                if let Some(arr) = self.heap.get_mut(heap_idx) {
                    let mut new_len = 0.0;
                    for arg in args {
                        new_len = arr.array_push(arg.clone());
                    }
                    Ok(Value::Number(new_len))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "pop" => {
                if let Some(arr) = self.heap.get_mut(heap_idx) {
                    Ok(arr.array_pop())
                } else {
                    Ok(Value::Undefined)
                }
            }
            "shift" => {
                if let Some(arr) = self.heap.get_mut(heap_idx) {
                    Ok(arr.array_shift())
                } else {
                    Ok(Value::Undefined)
                }
            }
            "unshift" => {
                if let Some(arr) = self.heap.get_mut(heap_idx) {
                    let mut new_len = 0.0;
                    // Insert in reverse order to maintain arg order
                    for arg in args.iter().rev() {
                        new_len = arr.array_unshift(arg.clone());
                    }
                    Ok(Value::Number(new_len))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "join" => {
                let separator = args.first()
                    .map(|v| v.to_js_string())
                    .unwrap_or_else(|| ",".to_string());
                if let Some(arr) = self.heap.get(heap_idx) {
                    Ok(Value::String(arr.array_join(&separator)))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            "reverse" => {
                if let Some(arr) = self.heap.get_mut(heap_idx) {
                    arr.array_reverse();
                }
                Ok(Value::Object(heap_idx))
            }
            "slice" => {
                let start = args.first().map(|v| v.to_number() as i32).unwrap_or(0);
                let end = args.get(1).map(|v| v.to_number() as i32).unwrap_or(i32::MAX);
                if let Some(arr) = self.heap.get(heap_idx) {
                    let elements = arr.array_slice(start, end);
                    let new_idx = self.alloc_object(RuntimeObject::new_array(elements));
                    Ok(Value::Object(new_idx))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "indexOf" => {
                let search = args.first().unwrap_or(&Value::Undefined);
                if let Some(arr) = self.heap.get(heap_idx) {
                    Ok(Value::Number(arr.array_index_of(search) as f64))
                } else {
                    Ok(Value::Number(-1.0))
                }
            }
            "concat" => {
                // Flatten array arguments
                let mut concat_elements = Vec::new();
                for arg in args {
                    if let Value::Object(idx) = arg
                        && let Some(other_arr) = self.heap.get(*idx)
                            && other_arr.is_array() {
                                concat_elements.extend(other_arr.elements().iter().cloned());
                                continue;
                            }
                    concat_elements.push(arg.clone());
                }
                if let Some(arr) = self.heap.get(heap_idx) {
                    let elements = arr.array_concat(&concat_elements);
                    let new_idx = self.alloc_object(RuntimeObject::new_array(elements));
                    Ok(Value::Object(new_idx))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "toString" | "toLocaleString" => {
                if let Some(arr) = self.heap.get(heap_idx) {
                    Ok(Value::String(arr.array_join(",")))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            "sort" => {
                // Basic string-based sort (ES3 default)
                if let Some(arr) = self.heap.get_mut(heap_idx)
                    && arr.is_array {
                        arr.array_elements.sort_by(|a, b| {
                            a.to_js_string().cmp(&b.to_js_string())
                        });
                    }
                Ok(Value::Object(heap_idx))
            }
            "lastIndexOf" => {
                let search = args.first().unwrap_or(&Value::Undefined);
                if let Some(arr) = self.heap.get(heap_idx)
                    && arr.is_array() {
                        for (i, elem) in arr.elements().iter().enumerate().rev() {
                            if elem == search {
                                return Ok(Value::Number(i as f64));
                            }
                        }
                    }
                Ok(Value::Number(-1.0))
            }
            "splice" => {
                // Get array length first
                let len = if let Some(arr) = self.heap.get(heap_idx) {
                    if arr.is_array() {
                        arr.array_elements.len() as i32
                    } else {
                        0
                    }
                } else {
                    0
                };

                // Calculate start index
                let start = args.first().map(|v| {
                    let n = v.to_integer() as i32;
                    if n < 0 {
                        (len + n).max(0) as usize
                    } else {
                        n.min(len) as usize
                    }
                }).unwrap_or(0);

                // Calculate delete count
                let delete_count = args.get(1).map(|v| {
                    let n = v.to_integer() as i32;
                    n.max(0).min(len - start as i32) as usize
                }).unwrap_or((len - start as i32).max(0) as usize);

                // Items to insert
                let items: Vec<Value> = args.iter().skip(2).cloned().collect();

                // Perform splice operation
                let removed = if let Some(arr) = self.heap.get_mut(heap_idx) {
                    if arr.is_array() {
                        let end = (start + delete_count).min(arr.array_elements.len());
                        let removed: Vec<Value> = arr.array_elements.drain(start..end).collect();

                        // Insert new items
                        for (i, item) in items.into_iter().enumerate() {
                            arr.array_elements.insert(start + i, item);
                        }

                        removed
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                // Return removed elements as new array
                let new_idx = self.alloc_object(RuntimeObject::new_array(removed));
                Ok(Value::Object(new_idx))
            }
            "length" => {
                if let Some(arr) = self.heap.get(heap_idx)
                    && arr.is_array() {
                        return Ok(Value::Number(arr.array_elements.len() as f64));
                    }
                Ok(Value::Number(0.0))
            }
            _ => Err(Error::TypeError(format!("Array method '{}' not implemented", method))),
        }
    }

    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        self.ip = 0;
        self.stack.clear();
        self.locals.clear();

        loop {
            if self.ip >= bytecode.instructions.len() {
                break;
            }

            let instruction = &bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Halt => break,

                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = bytecode.constants[*idx as usize].clone();
                        // If loading a function, capture the current closure environment
                        let value = if let Value::Function(callable) = &value {
                            if let Callable::Function(func) = callable.as_ref() {
                                // Clone the function with the current scope's variables as closure env
                                let mut closure_env = std::collections::HashMap::new();
                                // Capture current locals by name (we need to track local names)
                                // For now, we'll use globals which are accessible by name
                                for (name, val) in &self.globals {
                                    closure_env.insert(name.clone(), val.clone());
                                }
                                let mut new_func = func.clone();
                                new_func.closure_env = closure_env;
                                Value::Function(Arc::new(Callable::Function(new_func)))
                            } else {
                                value
                            }
                        } else {
                            value
                        };
                        self.stack.push(value);
                    }
                }

                OpCode::LoadUndefined => self.stack.push(Value::Undefined),
                OpCode::LoadNull => self.stack.push(Value::Null),
                OpCode::LoadTrue => self.stack.push(Value::Boolean(true)),
                OpCode::LoadFalse => self.stack.push(Value::Boolean(false)),

                // Local variable operations
                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self
                            .locals
                            .get(*idx as usize)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        let idx = *idx as usize;
                        if idx >= self.locals.len() {
                            self.locals.resize(idx + 1, Value::Undefined);
                        }
                        self.locals[idx] = value;
                    }
                }

                // Global variable operations
                OpCode::LoadGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand
                        && let Value::String(name) = &bytecode.constants[*idx as usize] {
                            let value = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                            self.stack.push(value);
                        }
                }

                OpCode::StoreGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand
                        && let Value::String(name) = &bytecode.constants[*idx as usize] {
                            let value = self.pop()?;
                            self.globals.insert(name.clone(), value);
                        }
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                OpCode::Swap => {
                    let len = self.stack.len();
                    if len >= 2 {
                        self.stack.swap(len - 1, len - 2);
                    }
                }

                // Arithmetic
                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,

                OpCode::Neg => {
                    if let Some(Value::Number(n)) = self.stack.pop() {
                        self.stack.push(Value::Number(-n));
                    }
                }

                // Comparison
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Le => self.compare_op(|a, b| a <= b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Ge => self.compare_op(|a, b| a >= b)?,

                OpCode::Eq => {
                    // Abstract equality (ES3 Section 11.9.3)
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(abstract_equals(&a, &b)));
                }
                OpCode::StrictEq => {
                    // Strict equality (ES3 Section 11.9.6)
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Ne => {
                    // Abstract inequality
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(!abstract_equals(&a, &b)));
                }
                OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a != b));
                }

                OpCode::Not => {
                    let value = self.pop()?;
                    self.stack.push(Value::Boolean(!value.to_boolean()));
                }

                // Bitwise
                OpCode::BitNot => {
                    let value = self.pop()?;
                    if let Value::Number(n) = value {
                        self.stack.push(Value::Number(!(n as i32) as f64));
                    } else {
                        return Err(Error::TypeError("Expected number".into()));
                    }
                }

                OpCode::BitAnd => self.bitwise_op(|a, b| a & b)?,
                OpCode::BitOr => self.bitwise_op(|a, b| a | b)?,
                OpCode::BitXor => self.bitwise_op(|a, b| a ^ b)?,
                OpCode::Shl => self.bitwise_op(|a, b| a << (b & 0x1f))?,
                OpCode::Shr => self.bitwise_op(|a, b| a >> (b & 0x1f))?,
                OpCode::Ushr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    // Use ToUint32 for proper unsigned right shift semantics
                    let a_u32 = a.to_uint32();
                    let b_u32 = b.to_uint32();
                    let result = a_u32 >> (b_u32 & 0x1f);
                    self.stack.push(Value::Number(result as f64));
                }

                // Control flow
                OpCode::Jump => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        self.ip = *target as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if !condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                OpCode::JumpIfTrue => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                // Object/array operations
                OpCode::NewArray => {
                    if let Some(Operand::ArgCount(count)) = &instruction.operand {
                        let count = *count as usize;
                        // Pop elements from stack (in reverse order)
                        let mut elements = Vec::with_capacity(count);
                        for _ in 0..count {
                            elements.push(self.pop()?);
                        }
                        elements.reverse();
                        // Create array on heap
                        let idx = self.alloc_object(RuntimeObject::new_array(elements));
                        self.stack.push(Value::Object(idx));
                    } else {
                        // Empty array
                        let idx = self.alloc_object(RuntimeObject::new_array(vec![]));
                        self.stack.push(Value::Object(idx));
                    }
                }

                OpCode::NewObject => {
                    let idx = self.alloc_object(RuntimeObject::new());
                    self.stack.push(Value::Object(idx));
                }

                OpCode::GetProperty => {
                    // Get property name from operand or stack
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        // Property name is in constants
                        match bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        // Property name/index is on stack
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    // Get object from stack
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Access property based on value type
                    let result = match &obj {
                        Value::Number(n) => {
                            // Number properties
                            match prop_name.as_str() {
                                "toString" | "toFixed" | "toExponential" | "toPrecision" | "valueOf" => {
                                    Value::String(format!("__number_method__{}:{}", prop_name, n))
                                }
                                _ => Value::Undefined,
                            }
                        }
                        Value::String(s) => {
                            // String properties
                            match prop_name.as_str() {
                                "length" => Value::Number(s.len() as f64),
                                // String prototype methods
                                "charAt" | "charCodeAt" | "indexOf" | "lastIndexOf" |
                                "substring" | "slice" | "substr" | "toLowerCase" | "toUpperCase" |
                                "split" | "trim" | "replace" | "concat" | "toString" | "valueOf" |
                                "match" | "search" => {
                                    Value::String(format!("__string_method__{}:{}", prop_name, s))
                                }
                                _ => {
                                    // Numeric index access
                                    if let Ok(idx) = prop_name.parse::<usize>() {
                                        s.chars()
                                            .nth(idx)
                                            .map(|c| Value::String(c.to_string()))
                                            .unwrap_or(Value::Undefined)
                                    } else {
                                        Value::Undefined
                                    }
                                }
                            }
                        }
                        Value::Object(heap_idx) => {
                            // Object property access from heap
                            if let Some(obj) = self.heap.get(*heap_idx) {
                                // Check for array methods
                                if obj.is_array() {
                                    match prop_name.as_str() {
                                        "push" | "pop" | "shift" | "unshift" | "splice" | "slice" |
                                        "concat" | "join" | "reverse" | "sort" | "indexOf" | "lastIndexOf" |
                                        "toString" | "toLocaleString" => {
                                            Value::String(format!("__array_method__{}_{}", prop_name, heap_idx))
                                        }
                                        _ => obj.get(&prop_name)
                                    }
                                } else {
                                    // Check if this is a Date object
                                    if let Value::String(type_str) = obj.get("__type__") {
                                        if type_str == "Date" {
                                            match prop_name.as_str() {
                                                "getTime" | "getFullYear" | "getMonth" | "getDate" |
                                                "getDay" | "getHours" | "getMinutes" | "getSeconds" |
                                                "getMilliseconds" | "toString" | "toDateString" |
                                                "toTimeString" | "valueOf" => {
                                                    // Return a marker for Date method
                                                    let timestamp = if let Value::Number(ts) = obj.get("__timestamp__") {
                                                        ts
                                                    } else {
                                                        f64::NAN
                                                    };
                                                    Value::String(format!("__date_method__{}:{}", prop_name, timestamp))
                                                }
                                                _ => obj.get(&prop_name)
                                            }
                                        } else {
                                            obj.get(&prop_name)
                                        }
                                    } else {
                                        obj.get(&prop_name)
                                    }
                                }
                            } else {
                                Value::Undefined
                            }
                        }
                        Value::NativeObject(props) => {
                            // Check if this is a RegExp object
                            if let Some(Value::String(type_str)) = props.get("__type__") {
                                if type_str == "RegExp" {
                                    match prop_name.as_str() {
                                        "test" | "exec" | "toString" => {
                                            // Return a marker for RegExp method
                                            let regex_str = props
                                                .get("__regex__")
                                                .map(|v| v.to_js_string())
                                                .unwrap_or_default();
                                            Value::String(format!("__regexp_method__{}:{}", prop_name, regex_str))
                                        }
                                        _ => props.get(&prop_name).cloned().unwrap_or(Value::Undefined),
                                    }
                                } else {
                                    props.get(&prop_name).cloned().unwrap_or(Value::Undefined)
                                }
                            } else {
                                // Regular native object property access (e.g., console.log, Math.abs)
                                props.get(&prop_name).cloned().unwrap_or(Value::Undefined)
                            }
                        }
                        _ => Value::Undefined,
                    };

                    self.stack.push(result);
                }

                OpCode::SetProperty => {
                    // Get property name from operand or stack
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        match bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    // Get value to set
                    let value = self.pop()?;

                    // Get object
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Set property on object in heap
                    if let Value::Object(idx) = obj {
                        if let Some(obj) = self.heap.get_mut(idx) {
                            obj.set(&prop_name, value.clone());
                        }
                        // Push the object back (object is still on stack for chained property access)
                        self.stack.push(Value::Object(idx));
                        // Also push the value (result of assignment)
                        self.stack.push(value);
                    } else {
                        self.stack.push(Value::Undefined);
                    }
                }

                OpCode::DeleteProperty => {
                    // Get property name from operand or stack
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        match bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Boolean(false));
                                continue;
                            }
                        }
                    } else {
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Boolean(false));
                                continue;
                            }
                        }
                    };

                    // Get object
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Delete property from object in heap
                    if let Value::Object(idx) = obj {
                        if let Some(obj) = self.heap.get_mut(idx) {
                            obj.delete(&prop_name);
                        }
                        self.stack.push(Value::Boolean(true));
                    } else {
                        self.stack.push(Value::Boolean(false));
                    }
                }

                OpCode::LoadThis => {
                    // TODO: Implement proper 'this' binding
                    self.stack.push(Value::Undefined);
                }

                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;

                        // Collect arguments from stack
                        let mut args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            args.push(self.pop()?);
                        }
                        args.reverse(); // Arguments were pushed in order

                        // Get callee
                        let callee = self.pop()?;
                        let callee_for_named = callee.clone(); // Clone for named function expressions

                        // Check for array method marker (__array_method__METHOD_HEAPIDX)
                        if let Value::String(s) = &callee {
                            if let Some(rest) = s.strip_prefix("__array_method__")
                                && let Some(last_underscore) = rest.rfind('_') {
                                    let method = &rest[..last_underscore];
                                    let heap_idx: usize = rest[last_underscore + 1..].parse().unwrap_or(0);
                                    let result = self.call_array_method(heap_idx, method, &args)?;
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for string method marker (__string_method__METHOD:STRING)
                            if let Some(rest) = s.strip_prefix("__string_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let string_val = &rest[colon_pos + 1..];
                                    // Handle split specially to create an actual array
                                    if method == "split" {
                                        let separator = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                                        let parts: Vec<Value> = if separator.is_empty() {
                                            string_val.chars().map(|c| Value::String(c.to_string())).collect()
                                        } else {
                                            string_val.split(&separator).map(|p| Value::String(p.to_string())).collect()
                                        };
                                        let arr_idx = self.alloc_object(RuntimeObject::new_array(parts));
                                        self.stack.push(Value::Object(arr_idx));
                                    } else {
                                        let result = call_string_method(string_val, method, &args);
                                        self.stack.push(result);
                                    }
                                    continue;
                                }
                            // Check for number method marker (__number_method__METHOD:NUMBER)
                            if let Some(rest) = s.strip_prefix("__number_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let num_val: f64 = rest[colon_pos + 1..].parse().unwrap_or(f64::NAN);
                                    let result = call_number_method(num_val, method, &args);
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for date method marker (__date_method__METHOD:TIMESTAMP)
                            if let Some(rest) = s.strip_prefix("__date_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let timestamp: f64 = rest[colon_pos + 1..].parse().unwrap_or(f64::NAN);
                                    let result = call_date_method(timestamp, method, &args);
                                    self.stack.push(result);
                                    continue;
                                }
                            // Check for regexp method marker (__regexp_method__METHOD:/pattern/flags)
                            if let Some(rest) = s.strip_prefix("__regexp_method__")
                                && let Some(colon_pos) = rest.find(':') {
                                    let method = &rest[..colon_pos];
                                    let regex_str = &rest[colon_pos + 1..];
                                    let result = call_regexp_method(regex_str, method, &args);
                                    self.stack.push(result);
                                    continue;
                                }
                        }

                        match callee {
                            Value::NativeObject(props) => {
                                // NativeObject being called as constructor (e.g., new Date(), Number())

                                // First check for a "constructor" property (generic approach)
                                if let Some(constructor) = props.get("constructor")
                                    && let Value::Function(callable) = constructor
                                        && let Callable::Native { func, .. } = callable.as_ref() {
                                            let temp_func = Function::new(None, vec![], Bytecode::new(), 0);
                                            let mut frame = CallFrame::new(temp_func, 0);
                                            match func(&mut frame, &args) {
                                                Ok(result) => {
                                                    self.stack.push(result);
                                                    continue;
                                                }
                                                Err(e) => return Err(Error::TypeError(e)),
                                            }
                                        }

                                // Check for Date constructor (legacy approach for Date)
                                if props.contains_key("now") {
                                    // This is Date, call the Date constructor
                                    if let Some(constructor_fn) = self.globals.get("Date_constructor")
                                        && let Value::Function(callable) = constructor_fn
                                            && let Callable::Native { func, .. } = callable.as_ref() {
                                                let temp_func = Function::new(None, vec![], Bytecode::new(), 0);
                                                let mut frame = CallFrame::new(temp_func, 0);
                                                match func(&mut frame, &args) {
                                                    Ok(Value::Number(timestamp)) => {
                                                        // Create a Date object with the timestamp
                                                        let mut date_obj = RuntimeObject::new();
                                                        date_obj.set("__type__", Value::String("Date".to_string()));
                                                        date_obj.set("__timestamp__", Value::Number(timestamp));
                                                        let idx = self.alloc_object(date_obj);
                                                        self.stack.push(Value::Object(idx));
                                                        continue;
                                                    }
                                                    Ok(result) => self.stack.push(result),
                                                    Err(e) => return Err(Error::TypeError(e)),
                                                }
                                                continue;
                                            }
                                }
                                return Err(Error::TypeError("Value is not callable".into()));
                            }
                            Value::Function(callable) => {
                                match callable.as_ref() {
                                    Callable::Native { func, .. } => {
                                        // Create a temporary call frame for native function
                                        let temp_func =
                                            Function::new(None, vec![], Bytecode::new(), 0);
                                        let mut frame = CallFrame::new(temp_func, 0);

                                        // Call native function
                                        match func(&mut frame, &args) {
                                            Ok(result) => self.stack.push(result),
                                            Err(e) => return Err(Error::TypeError(e)),
                                        }
                                    }
                                    Callable::Function(func) => {
                                        // Execute the user-defined function
                                        // Pass the function value for named function expressions
                                        let result = self.call_function_with_value(
                                            func,
                                            &args,
                                            Some(callee_for_named),
                                        )?;
                                        self.stack.push(result);
                                    }
                                }
                            }
                            _ => {
                                return Err(Error::TypeError("Value is not callable".into()));
                            }
                        }
                    }
                }

                OpCode::Return => {
                    return self.pop();
                }

                OpCode::MakeClosure => {
                    // Pop variable names string from stack
                    let var_names_value = self.pop()?;
                    let var_names_str = match var_names_value {
                        Value::String(s) => s,
                        _ => String::new(),
                    };

                    // Pop the base function from stack
                    let base_func = self.pop()?;

                    // Create closure environment by capturing variables
                    let mut closure_env = std::collections::HashMap::new();
                    if !var_names_str.is_empty() {
                        for var_name in var_names_str.split(',') {
                            let var_name = var_name.trim();
                            if !var_name.is_empty() {
                                // Look up the variable in globals (where outer scope vars are stored)
                                let value = self.globals.get(var_name).cloned().unwrap_or(Value::Undefined);
                                closure_env.insert(var_name.to_string(), value);
                            }
                        }
                    }

                    // Create new function with closure environment
                    if let Value::Function(callable) = base_func {
                        match callable.as_ref() {
                            Callable::Function(func) => {
                                let new_func = Function::new_with_closure(
                                    func.name.clone(),
                                    func.params.clone(),
                                    func.bytecode.clone(),
                                    func.local_count,
                                    closure_env,
                                );
                                let new_callable = Callable::Function(new_func);
                                self.stack.push(Value::Function(Arc::new(new_callable)));
                            }
                            _ => {
                                // Native functions don't support closures, just push as-is
                                self.stack.push(Value::Function(callable));
                            }
                        }
                    } else {
                        self.stack.push(base_func);
                    }
                }

                OpCode::Nop => {}

                OpCode::ForInInit => {
                    // Pop object to iterate, collect keys, push iteration state
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Get enumerable keys from the object
                    let keys: Vec<String> = match &obj {
                        Value::Object(idx) => {
                            // Get keys from heap object
                            if let Some(heap_obj) = self.heap.get(*idx) {
                                heap_obj.keys()
                            } else {
                                vec![]
                            }
                        }
                        Value::NativeObject(props) => {
                            // Get keys from native object
                            props.keys().cloned().collect()
                        }
                        Value::String(s) => {
                            // String indices as keys
                            (0..s.len()).map(|i| i.to_string()).collect()
                        }
                        _ => vec![],
                    };

                    // Store keys in a new heap object for iteration
                    let keys_obj_idx = self.alloc_object(RuntimeObject::new());
                    if let Some(keys_obj) = self.heap.get_mut(keys_obj_idx) {
                        for (i, key) in keys.iter().enumerate() {
                            keys_obj.set(&i.to_string(), Value::String(key.clone()));
                        }
                        keys_obj.set("length", Value::Number(keys.len() as f64));
                    }

                    // Push iteration state: keys object, current index
                    self.stack.push(Value::Object(keys_obj_idx));
                    self.stack.push(Value::Number(0.0)); // Current index
                }

                OpCode::ForInNext => {
                    // Check if there are more keys
                    // Stack: [keys_obj, index]
                    if let Some(Operand::Jump(target)) = instruction.operand {
                        let index = match self.stack.pop() {
                            Some(Value::Number(n)) => n as usize,
                            _ => 0,
                        };
                        let keys_obj = self.stack.pop().unwrap_or(Value::Undefined);

                        let count = match &keys_obj {
                            Value::Object(idx) => {
                                if let Some(obj) = self.heap.get(*idx) {
                                    match obj.get("length") {
                                        Value::Number(n) => n as usize,
                                        _ => 0,
                                    }
                                } else {
                                    0
                                }
                            }
                            _ => 0,
                        };

                        if index >= count {
                            // No more keys, jump to end
                            self.ip = target as usize;
                            // Push dummy values to keep stack balanced for ForInDone
                            self.stack.push(keys_obj);
                            self.stack.push(Value::Number(index as f64));
                        } else {
                            // Get the key at current index
                            let key = match &keys_obj {
                                Value::Object(idx) => {
                                    if let Some(obj) = self.heap.get(*idx) {
                                        obj.get(&index.to_string())
                                    } else {
                                        Value::Undefined
                                    }
                                }
                                _ => Value::Undefined,
                            };

                            // Restore iteration state with incremented index
                            self.stack.push(keys_obj);
                            self.stack.push(Value::Number((index + 1) as f64));
                            // Push the key value for the loop body to use
                            self.stack.push(key);
                        }
                    }
                }

                OpCode::ForInDone => {
                    // Clean up iteration state
                    // Stack should have: [keys_obj, index]
                    self.stack.pop(); // index
                    self.stack.pop(); // keys_obj
                }

                OpCode::LogicalAnd => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Short-circuit AND: return left if falsy, else right
                    if left.to_boolean() {
                        self.stack.push(right);
                    } else {
                        self.stack.push(left);
                    }
                }

                OpCode::LogicalOr => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Short-circuit OR: return left if truthy, else right
                    if left.to_boolean() {
                        self.stack.push(left);
                    } else {
                        self.stack.push(right);
                    }
                }

                OpCode::TypeOf => {
                    let val = self.stack.pop().unwrap_or(Value::Undefined);
                    self.stack.push(Value::String(val.type_of().to_string()));
                }

                OpCode::InstanceOf => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Simplified instanceof - checks if left is an object
                    let result = matches!(
                        (&left, &right),
                        (Value::Object(_), Value::Function(_))
                    );
                    self.stack.push(Value::Boolean(result));
                }

                OpCode::In => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);

                    // The 'in' operator checks if a property exists on an object
                    let result = match (&left, &right) {
                        (Value::String(prop), Value::Object(idx)) => {
                            // Check if property exists on the object
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(prop) != Value::Undefined
                            } else {
                                false
                            }
                        }
                        (Value::Number(n), Value::Object(idx)) => {
                            // Check numeric index
                            let prop = n.to_string();
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(&prop) != Value::Undefined
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    self.stack.push(Value::Boolean(result));
                }

                _ => {
                    // TODO: Implement remaining opcodes
                }
            }
        }

        // Return the top of stack, or undefined if empty
        Ok(self.stack.pop().unwrap_or(Value::Undefined))
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack
            .pop()
            .ok_or_else(|| Error::InternalError("Stack underflow".into()))
    }

    fn binary_add(&mut self) -> Result<(), Error> {
        let b = self.pop()?;
        let a = self.pop()?;

        let result = match (&a, &b) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (Value::String(a), b) => Value::String(format!("{}{}", a, b)),
            (a, Value::String(b)) => Value::String(format!("{}{}", a, b)),
            _ => Value::Number(f64::NAN),
        };

        self.stack.push(result);
        Ok(())
    }

    fn binary_num_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        // ES3: Coerce operands to numbers (returns NaN for non-numeric values)
        let a_num = a.to_number();
        let b_num = b.to_number();
        self.stack.push(Value::Number(op(a_num, b_num)));

        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        // ES3: Coerce operands to numbers for numeric comparison
        let a_num = a.to_number();
        let b_num = b.to_number();
        self.stack.push(Value::Boolean(op(a_num, b_num)));

        Ok(())
    }

    fn bitwise_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(i32, i32) -> i32,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        // ES3: Coerce operands to int32 for bitwise operations
        let a_int = a.to_int32();
        let b_int = b.to_int32();
        let result = op(a_int, b_int);
        self.stack.push(Value::Number(result as f64));

        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::parser::Parser;

    fn eval(src: &str) -> Result<Value, Error> {
        let mut parser = Parser::new(src);
        let program = parser.parse_program()?;
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program)?;
        let mut vm = VM::new();
        vm.execute(&bytecode)
    }

    fn eval_ok(src: &str) -> Value {
        eval(src).expect("Evaluation should succeed")
    }

    #[test]
    fn test_vm_new() {
        let vm = VM::new();
        assert!(vm.stack.is_empty());
        assert!(vm.locals.is_empty());
    }

    #[test]
    fn test_vm_default() {
        let vm = VM::default();
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_eval_number() {
        let result = eval_ok("42;");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_float() {
        let result = eval_ok("3.14;");
        assert!(matches!(result, Value::Number(n) if (n - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_eval_string() {
        let result = eval_ok("'hello';");
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_eval_boolean_true() {
        let result = eval_ok("true;");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_eval_boolean_false() {
        let result = eval_ok("false;");
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eval_null() {
        let result = eval_ok("null;");
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_eval_add_numbers() {
        let result = eval_ok("1 + 2;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_subtract() {
        let result = eval_ok("5 - 3;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_multiply() {
        let result = eval_ok("4 * 5;");
        assert!(matches!(result, Value::Number(n) if n == 20.0));
    }

    #[test]
    fn test_eval_divide() {
        let result = eval_ok("10 / 2;");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_eval_modulo() {
        let result = eval_ok("7 % 3;");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_eval_string_concat() {
        let result = eval_ok("'hello' + ' ' + 'world';");
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }

    #[test]
    fn test_eval_string_number_concat() {
        let result = eval_ok("'count: ' + 42;");
        assert!(matches!(result, Value::String(s) if s == "count: 42"));
    }

    #[test]
    fn test_eval_less_than() {
        assert!(matches!(eval_ok("1 < 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 < 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_greater_than() {
        assert!(matches!(eval_ok("2 > 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 > 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_less_than_equal() {
        assert!(matches!(eval_ok("1 <= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 <= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("3 <= 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_greater_than_equal() {
        assert!(matches!(eval_ok("2 >= 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 >= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 >= 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_equal() {
        assert!(matches!(eval_ok("1 == 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 == 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_not_equal() {
        assert!(matches!(eval_ok("1 != 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 != 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_strict_equal() {
        assert!(matches!(eval_ok("1 === 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 === 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_strict_not_equal() {
        assert!(matches!(eval_ok("1 !== 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 !== 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_negate() {
        let result = eval_ok("-42;");
        assert!(matches!(result, Value::Number(n) if n == -42.0));
    }

    #[test]
    fn test_eval_not() {
        assert!(matches!(eval_ok("!true;"), Value::Boolean(false)));
        assert!(matches!(eval_ok("!false;"), Value::Boolean(true)));
    }

    #[test]
    fn test_eval_variable() {
        let result = eval_ok("let x = 42; x;");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_variable_assignment() {
        let result = eval_ok("let x = 1; x = 2; x;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_multiple_variables() {
        let result = eval_ok("let a = 1; let b = 2; a + b;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_if_true() {
        let result = eval_ok("let x = 0; if (true) { x = 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_eval_if_false() {
        let result = eval_ok("let x = 0; if (false) { x = 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_eval_if_else() {
        let result = eval_ok("let x = 0; if (false) { x = 1; } else { x = 2; } x;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_while_loop() {
        let result = eval_ok("let x = 0; while (x < 3) { x = x + 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_for_loop() {
        let result =
            eval_ok("let sum = 0; for (let i = 0; i < 5; i = i + 1) { sum = sum + i; } sum;");
        assert!(matches!(result, Value::Number(n) if n == 10.0)); // 0+1+2+3+4
    }

    // Note: User-defined function calls are not yet fully supported
    // These tests are placeholders for when they are implemented

    #[test]
    fn test_eval_array_literal() {
        let result = eval_ok("let arr = [1, 2, 3]; arr;");
        // Array should be an object
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_eval_empty_program() {
        let result = eval_ok("");
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_eval_expression_precedence() {
        let result = eval_ok("2 + 3 * 4;");
        assert!(matches!(result, Value::Number(n) if n == 14.0)); // 2 + 12
    }

    #[test]
    fn test_vm_register_native() {
        let mut vm = VM::new();
        fn custom_fn(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
            Ok(Value::Number(999.0))
        }
        vm.register_native("custom", 0, custom_fn);
        assert!(vm.get_native("custom").is_some());
    }

    #[test]
    fn test_vm_get_native_not_found() {
        let vm = VM::new();
        assert!(vm.get_native("nonexistent").is_none());
    }

    #[test]
    fn test_eval_complex_expression() {
        let result = eval_ok("let x = 5; let y = 3; x * y + 2;");
        assert!(matches!(result, Value::Number(n) if n == 17.0));
    }
}

