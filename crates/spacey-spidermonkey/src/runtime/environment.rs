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
            && binding.initialized {
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
            && binding.mutable && binding.initialized {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_environment() {
        let env = Environment::new();
        assert!(env.get("x").is_none());
    }

    #[test]
    fn test_declare_and_initialize() {
        let mut env = Environment::new();

        // Declare a mutable variable
        env.declare("x".to_string(), true);

        // Variable is declared but not initialized
        assert!(env.get("x").is_none());

        // Initialize the variable
        assert!(env.initialize("x", Value::Number(42.0)));

        // Now we can get the value
        assert_eq!(env.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_initialize_undeclared() {
        let mut env = Environment::new();

        // Cannot initialize undeclared variable
        assert!(!env.initialize("x", Value::Number(42.0)));
    }

    #[test]
    fn test_set_mutable() {
        let mut env = Environment::new();

        env.declare("x".to_string(), true);
        env.initialize("x", Value::Number(1.0));

        // Can set mutable variable
        assert!(env.set("x", Value::Number(2.0)));
        assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_set_immutable() {
        let mut env = Environment::new();

        env.declare("x".to_string(), false); // const
        env.initialize("x", Value::Number(1.0));

        // Cannot set immutable variable
        assert!(!env.set("x", Value::Number(2.0)));
        assert_eq!(env.get("x"), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_set_uninitialized() {
        let mut env = Environment::new();

        env.declare("x".to_string(), true);

        // Cannot set uninitialized variable
        assert!(!env.set("x", Value::Number(2.0)));
    }

    #[test]
    fn test_set_undeclared() {
        let mut env = Environment::new();

        // Cannot set undeclared variable
        assert!(!env.set("x", Value::Number(2.0)));
    }

    #[test]
    fn test_with_outer_environment() {
        let mut outer = Environment::new();
        outer.declare("x".to_string(), true);
        outer.initialize("x", Value::Number(10.0));

        let inner = Environment::with_outer(outer);

        // Can access outer variable from inner
        assert_eq!(inner.get("x"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn test_inner_shadows_outer() {
        let mut outer = Environment::new();
        outer.declare("x".to_string(), true);
        outer.initialize("x", Value::Number(10.0));

        let mut inner = Environment::with_outer(outer);
        inner.declare("x".to_string(), true);
        inner.initialize("x", Value::Number(20.0));

        // Inner shadows outer
        assert_eq!(inner.get("x"), Some(&Value::Number(20.0)));
    }

    #[test]
    fn test_set_outer_variable() {
        let mut outer = Environment::new();
        outer.declare("x".to_string(), true);
        outer.initialize("x", Value::Number(10.0));

        let mut inner = Environment::with_outer(outer);

        // Can set outer variable from inner
        assert!(inner.set("x", Value::Number(20.0)));
        assert_eq!(inner.get("x"), Some(&Value::Number(20.0)));
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert!(env.get("anything").is_none());
    }

    #[test]
    fn test_environment_clone() {
        let mut env = Environment::new();
        env.declare("x".to_string(), true);
        env.initialize("x", Value::Number(42.0));

        let cloned = env.clone();
        assert_eq!(cloned.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_multiple_variables() {
        let mut env = Environment::new();

        env.declare("a".to_string(), true);
        env.declare("b".to_string(), false);
        env.declare("c".to_string(), true);

        env.initialize("a", Value::Number(1.0));
        env.initialize("b", Value::String("hello".to_string()));
        env.initialize("c", Value::Boolean(true));

        assert_eq!(env.get("a"), Some(&Value::Number(1.0)));
        assert_eq!(env.get("b"), Some(&Value::String("hello".to_string())));
        assert_eq!(env.get("c"), Some(&Value::Boolean(true)));

        // Can modify mutable
        assert!(env.set("a", Value::Number(2.0)));
        assert!(env.set("c", Value::Boolean(false)));

        // Cannot modify immutable
        assert!(!env.set("b", Value::String("world".to_string())));
    }

    #[test]
    fn test_deeply_nested_environments() {
        let mut level0 = Environment::new();
        level0.declare("x".to_string(), true);
        level0.initialize("x", Value::Number(0.0));

        let mut level1 = Environment::with_outer(level0);
        level1.declare("y".to_string(), true);
        level1.initialize("y", Value::Number(1.0));

        let mut level2 = Environment::with_outer(level1);
        level2.declare("z".to_string(), true);
        level2.initialize("z", Value::Number(2.0));

        // Can access all levels
        assert_eq!(level2.get("x"), Some(&Value::Number(0.0)));
        assert_eq!(level2.get("y"), Some(&Value::Number(1.0)));
        assert_eq!(level2.get("z"), Some(&Value::Number(2.0)));
    }
}
