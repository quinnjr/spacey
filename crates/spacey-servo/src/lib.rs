//! Servo browser integration with Spacey JavaScript engine.
//!
//! This crate provides a bridge between the Servo browser engine and the Spacey
//! JavaScript engine, replacing Servo's default SpiderMonkey integration with
//! our custom implementation.
//!
//! # Architecture
//!
//! Servo expects a JavaScript engine that implements certain traits and provides
//! specific functionality:
//!
//! - **Script Runtime**: Manages the JavaScript execution context
//! - **DOM Bindings**: Provides JavaScript bindings for DOM objects
//! - **Event Loop Integration**: Handles async operations and microtasks
//! - **Memory Management**: Integrates with Servo's garbage collection
//!
//! # Usage
//!
//! ```ignore
//! use spacey_servo::SpaceyRuntime;
//!
//! // Create a Spacey-powered Servo instance
//! let runtime = SpaceyRuntime::new();
//! let servo = Servo::new(runtime);
//! ```

#![warn(missing_docs)]

use spacey_spidermonkey::Engine;
use std::sync::Arc;
use parking_lot::RwLock;

mod runtime;
mod bindings;
mod event_loop;

pub use runtime::SpaceyRuntime;
pub use bindings::DomBindings;
pub use event_loop::EventLoop;

/// The main Spacey-Servo integration struct.
///
/// This provides the glue between Servo's expectations and Spacey's
/// JavaScript engine implementation.
#[derive(Clone)]
pub struct SpaceyServo {
    engine: Arc<RwLock<Engine>>,
    event_loop: Arc<EventLoop>,
}

impl SpaceyServo {
    /// Create a new Spacey-Servo integration.
    ///
    /// # Example
    ///
    /// ```
    /// use spacey_servo::SpaceyServo;
    ///
    /// let servo = SpaceyServo::new();
    /// ```
    pub fn new() -> Self {
        let engine = Arc::new(RwLock::new(Engine::new()));
        let event_loop = Arc::new(EventLoop::new());

        Self {
            engine,
            event_loop,
        }
    }

    /// Get a reference to the JavaScript engine.
    pub fn engine(&self) -> Arc<RwLock<Engine>> {
        Arc::clone(&self.engine)
    }

    /// Get a reference to the event loop.
    pub fn event_loop(&self) -> Arc<EventLoop> {
        Arc::clone(&self.event_loop)
    }

    /// Execute JavaScript code in the Servo context.
    ///
    /// # Example
    ///
    /// ```
    /// use spacey_servo::SpaceyServo;
    ///
    /// let servo = SpaceyServo::new();
    /// let result = servo.eval("1 + 2;");
    /// assert!(result.is_ok());
    /// ```
    pub fn eval(&self, source: &str) -> Result<String, String> {
        let mut engine = self.engine.write();
        let value = engine.eval(source).map_err(|e| format!("{:?}", e))?;
        Ok(format!("{:?}", value))
    }

    /// Execute a script file.
    pub fn eval_file(&self, path: &str) -> Result<String, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        self.eval(&source)
    }
}

impl Default for SpaceyServo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_servo() {
        let servo = SpaceyServo::new();
        let result = servo.engine().write().eval("1 + 1;");
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval() {
        let servo = SpaceyServo::new();
        let result = servo.eval("2 + 2;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_evals() {
        let servo = SpaceyServo::new();
        
        servo.eval("var x = 10;").unwrap();
        servo.eval("var y = 20;").unwrap();
        let result = servo.eval("x + y;");
        
        assert!(result.is_ok());
    }
}
