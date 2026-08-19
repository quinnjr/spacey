// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Enhanced console object for Node.js compatibility

use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

/// Console timer tracking
static TIMERS: std::sync::LazyLock<parking_lot::Mutex<HashMap<String, Instant>>> =
    std::sync::LazyLock::new(|| parking_lot::Mutex::new(HashMap::new()));

/// Console count tracking
static COUNTERS: std::sync::LazyLock<parking_lot::Mutex<HashMap<String, u64>>> =
    std::sync::LazyLock::new(|| parking_lot::Mutex::new(HashMap::new()));

/// Create the console object
pub fn create_console_object() -> Value {
    let mut console = HashMap::new();

    // Basic output methods are implemented in the engine's builtins
    // Here we add Node.js-specific methods

    // console.time(label)
    // console.timeEnd(label)
    // console.timeLog(label)
    // console.count(label)
    // console.countReset(label)
    // console.assert(value, ...message)
    // console.dir(obj)
    // console.table(data)
    // console.group(label)
    // console.groupEnd()
    // console.groupCollapsed(label)
    // console.trace(message)
    // console.clear()

    Value::NativeObject(console)
}

/// Implementation of console.time
pub fn time(label: &str) {
    let mut timers = TIMERS.lock();
    timers.insert(label.to_string(), Instant::now());
}

/// Implementation of console.timeEnd
pub fn time_end(label: &str) {
    let timers = TIMERS.lock();
    if let Some(start) = timers.get(label) {
        let elapsed = start.elapsed();
        println!("{}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
    } else {
        eprintln!("Timer '{}' does not exist", label);
    }
}

/// Implementation of console.timeLog
pub fn time_log(label: &str, args: &[Value]) {
    let timers = TIMERS.lock();
    if let Some(start) = timers.get(label) {
        let elapsed = start.elapsed();
        print!("{}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
        for arg in args {
            print!(" {}", format_value(arg));
        }
        println!();
    } else {
        eprintln!("Timer '{}' does not exist", label);
    }
}

/// Implementation of console.count
pub fn count(label: &str) {
    let mut counters = COUNTERS.lock();
    let count = counters.entry(label.to_string()).or_insert(0);
    *count += 1;
    println!("{}: {}", label, count);
}

/// Implementation of console.countReset
pub fn count_reset(label: &str) {
    let mut counters = COUNTERS.lock();
    counters.remove(label);
}

/// Implementation of console.assert
pub fn assert(condition: bool, args: &[Value]) {
    if !condition {
        eprint!("Assertion failed:");
        for arg in args {
            eprint!(" {}", format_value(arg));
        }
        eprintln!();
    }
}

/// Implementation of console.dir
pub fn dir(obj: &Value, options: Option<&DirOptions>) {
    let depth = options.map(|o| o.depth).unwrap_or(2);
    let colors = options.map(|o| o.colors).unwrap_or(true);

    println!("{}", format_value_with_depth(obj, depth, colors));
}

/// Options for console.dir
pub struct DirOptions {
    /// Maximum depth to recurse
    pub depth: u32,
    /// Whether to use colors
    pub colors: bool,
    /// Show non-enumerable properties
    pub show_hidden: bool,
}

impl Default for DirOptions {
    fn default() -> Self {
        Self {
            depth: 2,
            colors: true,
            show_hidden: false,
        }
    }
}

/// Implementation of console.table
pub fn table(data: &Value) {
    // Simplified implementation - just prints the data
    // A full implementation would format as a table
    println!("{}", format_value(data));
}

/// Implementation of console.trace
pub fn trace(message: Option<&str>) {
    if let Some(msg) = message {
        eprintln!("Trace: {}", msg);
    } else {
        eprintln!("Trace");
    }
    // In a full implementation, we'd print a stack trace here
}

/// Implementation of console.clear
pub fn clear() {
    // ANSI escape codes to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::stdout().flush();
}

/// Current group indentation level
static GROUP_LEVEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Implementation of console.group
pub fn group(label: Option<&str>) {
    if let Some(l) = label {
        println!("{}", l);
    }
    GROUP_LEVEL.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Implementation of console.groupEnd
pub fn group_end() {
    let level = GROUP_LEVEL.load(std::sync::atomic::Ordering::SeqCst);
    if level > 0 {
        GROUP_LEVEL.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Get current indentation
pub fn get_indent() -> String {
    let level = GROUP_LEVEL.load(std::sync::atomic::Ordering::SeqCst);
    "  ".repeat(level as usize)
}

/// Format a value for console output
fn format_value(value: &Value) -> String {
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
        Value::String(s) => s.clone(),
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::BigInt(s) => format!("{}n", s),
        Value::Object(_) => "[Object]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::NativeObject(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
    }
}

/// Format a value with depth limit
fn format_value_with_depth(value: &Value, depth: u32, _colors: bool) -> String {
    if depth == 0 {
        return "[...]".to_string();
    }

    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("'{}'", s),
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::BigInt(s) => format!("{}n", s),
        Value::Object(_) => "[Object]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::NativeObject(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_with_depth(v, depth - 1, _colors)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
    }
}
