//! Lexical environments for variable binding.

use super::value::Value;
use rustc_hash::FxHashMap;

/// A lexical environment for variable bindings.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    /// The bindings in this environment
    bindings: FxHashMap<String, Binding>,
    /// The outer (parent) environment
    outer: Option<Box<Environment>>,
}

impl Environment {
    /// Creates a new global environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new environment with an outer environment.
    pub fn with_outer(outer: Environment) -> Self {
        Self {
            bindings: FxHashMap::default(),
            outer: Some(Box::new(outer)),
        }
    }

    /// Declares a variable.
    pub fn declare(&mut self, name: String, mutable: bool) {
        self.bindings.insert(
            name,
            Binding {
                value: Value::Undefined,
                mutable,
                initialized: false,
            },
        );
    }

    /// Initializes a variable.
    pub fn initialize(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.bindings.get_mut(name) {
            binding.value = value;
            binding.initialized = true;
            true
        } else {
            false
        }
    }

    /// Gets a variable's value.
    pub fn get(&self, name: &str) -> Option<&Value> {
        if let Some(binding) = self.bindings.get(name)
            && binding.initialized
        {
            return Some(&binding.value);
        }
        if let Some(outer) = &self.outer {
            return outer.get(name);
        }
        None
    }

    /// Sets a variable's value.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.bindings.get_mut(name)
            && binding.mutable
            && binding.initialized
        {
            binding.value = value;
            return true;
        }
        if let Some(outer) = &mut self.outer {
            return outer.set(name, value);
        }
        false
    }
}

/// A variable binding.
#[derive(Debug, Clone)]
struct Binding {
    /// The value
    value: Value,
    /// Whether the binding is mutable (let vs const)
    mutable: bool,
    /// Whether the binding has been initialized
    initialized: bool,
}
