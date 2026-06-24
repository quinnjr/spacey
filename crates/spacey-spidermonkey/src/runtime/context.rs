//! Execution context and realm.

use super::environment::Environment;

/// An execution context representing the current state of execution.
pub struct Context {
    /// The global environment
    pub global_env: Environment,
}

impl Context {
    /// Creates a new execution context.
    pub fn new() -> Self {
        Self {
            global_env: Environment::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
