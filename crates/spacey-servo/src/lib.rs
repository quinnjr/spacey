//! Spacey Servo - Rendering and JavaScript execution for Spacey Browser
//!
//! This crate provides:
//! - Servo-based web content rendering
//! - JavaScript execution via Spacey SpiderMonkey
//! - DOM manipulation and layout

use spacey_spidermonkey::{Engine, Value};

/// Spacey Servo engine combining rendering and JS execution
pub struct SpaceyServo {
    /// JavaScript engine
    engine: Engine,
}

impl SpaceyServo {
    /// Create a new SpaceyServo instance
    pub fn new() -> Self {
        let engine = Engine::new();
        
        log::debug!("SpaceyServo initialized");
        
        Self { engine }
    }

    /// Evaluate JavaScript code and return the result as a string
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        match self.engine.eval(code) {
            Ok(value) => Ok(value_to_string(&value)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Execute JavaScript code without returning a result
    pub fn execute(&mut self, code: &str) -> Result<(), String> {
        self.engine.eval(code)
            .map(|_| ())
            .map_err(|e| format!("{}", e))
    }

    /// Get the JavaScript engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get mutable JavaScript engine
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

impl Default for SpaceyServo {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a JavaScript value to a string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Object(_) => "[object Object]".to_string(),
        Value::Symbol(s) => format!("Symbol({})", s),
        Value::BigInt(s) => format!("{}n", s),
        Value::Function(_) => "[Function]".to_string(),
        Value::NativeFunction(_) => "[native code]".to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        Value::ParsedObject(_) => "[object Object]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        let mut servo = SpaceyServo::new();
        
        let result = servo.eval("1 + 2;").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_eval_string() {
        let mut servo = SpaceyServo::new();
        
        let result = servo.eval("'hello' + ' world';").unwrap();
        assert_eq!(result, "hello world");
    }
}
