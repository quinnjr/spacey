//! JavaScript runtime implementation for Servo integration.
//!
//! This module provides the runtime interface that Servo expects from a
//! JavaScript engine.

use spacey_spidermonkey::Engine;
use std::sync::Arc;
use parking_lot::RwLock;

/// The Spacey runtime for Servo.
///
/// This struct implements the runtime interface that Servo expects,
/// bridging between Servo's requirements and Spacey's capabilities.
pub struct SpaceyRuntime {
    engine: Arc<RwLock<Engine>>,
    global_scope: Option<String>,
}

impl SpaceyRuntime {
    /// Create a new Spacey runtime.
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(Engine::new())),
            global_scope: None,
        }
    }

    /// Initialize the global scope with DOM bindings.
    pub fn init_global_scope(&mut self) {
        // TODO: Set up DOM bindings (window, document, etc.)
        self.global_scope = Some("window".to_string());

        // Initialize basic DOM objects
        let _ = self.engine.write().eval(r#"
            // Basic DOM stubs for Servo integration
            if (typeof window === 'undefined') {
                var window = {};
            }
            if (typeof document === 'undefined') {
                var document = {
                    createElement: function(tag) {
                        return { tagName: tag, children: [] };
                    },
                    getElementById: function(id) {
                        return null;
                    },
                    querySelector: function(selector) {
                        return null;
                    }
                };
            }
            if (typeof console === 'undefined') {
                var console = {
                    log: function() {},
                    error: function() {},
                    warn: function() {},
                    info: function() {}
                };
            }
        "#);
    }

    /// Execute JavaScript in the runtime.
    pub fn execute(&self, source: &str) -> Result<String, String> {
        self.engine
            .write()
            .eval(source)
            .map(|v| format!("{:?}", v))
            .map_err(|e| format!("{:?}", e))
    }

    /// Get the engine instance.
    pub fn engine(&self) -> Arc<RwLock<Engine>> {
        Arc::clone(&self.engine)
    }

    /// Check if the runtime has a global scope initialized.
    pub fn has_global_scope(&self) -> bool {
        self.global_scope.is_some()
    }
}

impl Default for SpaceyRuntime {
    fn default() -> Self {
        let mut runtime = Self::new();
        runtime.init_global_scope();
        runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = SpaceyRuntime::new();
        assert!(!runtime.has_global_scope());
    }

    #[test]
    fn test_runtime_with_global_scope() {
        let runtime = SpaceyRuntime::default();
        assert!(runtime.has_global_scope());
    }

    #[test]
    fn test_execute_basic() {
        let runtime = SpaceyRuntime::default();
        let result = runtime.execute("1 + 1;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dom_stubs() {
        let runtime = SpaceyRuntime::default();

        // Test that basic DOM objects exist
        let result = runtime.execute("typeof window;");
        assert!(result.is_ok());

        let result = runtime.execute("typeof document;");
        assert!(result.is_ok());

        let result = runtime.execute("typeof console;");
        assert!(result.is_ok());
    }
}
