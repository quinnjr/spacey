//! WeakRef and FinalizationRegistry implementations.

use std::collections::HashMap;

/// A WeakRef wrapper.
/// Note: This is a simplified implementation. A full implementation would
/// require integration with the garbage collector.
#[derive(Debug, Clone)]
pub struct WeakRef {
    /// The target object reference (would be weak in full implementation)
    target: Option<usize>,
}

impl WeakRef {
    /// Creates a new WeakRef to the given target.
    pub fn new(target: usize) -> Self {
        Self {
            target: Some(target),
        }
    }

    /// Dereferences the WeakRef, returning the target if it's still alive.
    pub fn deref(&self) -> Option<usize> {
        self.target
    }

    /// Clears the reference (simulates GC collection).
    pub fn clear(&mut self) {
        self.target = None;
    }
}

/// A cleanup callback for FinalizationRegistry.
#[derive(Debug, Clone)]
pub struct CleanupCallback {
    /// The callback function reference
    pub callback: usize,
    /// The held value to pass to the callback
    pub held_value: Option<usize>,
}

/// A FinalizationRegistry for registering cleanup callbacks.
#[derive(Debug, Default)]
pub struct FinalizationRegistry {
    /// The cleanup callback function
    cleanup_callback: Option<usize>,
    /// Registered targets and their held values
    registrations: HashMap<usize, RegistrationEntry>,
    /// Unregistration tokens
    tokens: HashMap<usize, usize>, // token -> target
}

/// A registration entry in the FinalizationRegistry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RegistrationEntry {
    /// The held value to pass to the cleanup callback
    held_value: Option<usize>,
    /// Optional unregister token
    unregister_token: Option<usize>,
}

impl FinalizationRegistry {
    /// Creates a new FinalizationRegistry with the given cleanup callback.
    pub fn new(cleanup_callback: usize) -> Self {
        Self {
            cleanup_callback: Some(cleanup_callback),
            registrations: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    /// Registers a target with an optional held value and unregister token.
    pub fn register(
        &mut self,
        target: usize,
        held_value: Option<usize>,
        unregister_token: Option<usize>,
    ) {
        self.registrations.insert(
            target,
            RegistrationEntry {
                held_value,
                unregister_token,
            },
        );

        if let Some(token) = unregister_token {
            self.tokens.insert(token, target);
        }
    }

    /// Unregisters a target using its unregister token.
    pub fn unregister(&mut self, unregister_token: usize) -> bool {
        if let Some(target) = self.tokens.remove(&unregister_token) {
            self.registrations.remove(&target);
            true
        } else {
            false
        }
    }

    /// Gets the cleanup callback.
    pub fn cleanup_callback(&self) -> Option<usize> {
        self.cleanup_callback
    }

    /// Simulates cleanup for a collected target.
    /// Returns the held value if the target was registered.
    pub fn notify_collected(&mut self, target: usize) -> Option<usize> {
        self.registrations
            .remove(&target)
            .and_then(|entry| entry.held_value)
    }
}

/// Global object reference (globalThis).
#[derive(Debug, Clone, Default)]
pub struct GlobalThis {
    /// Properties of the global object
    properties: HashMap<String, usize>,
}

impl GlobalThis {
    /// Creates a new global object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a property from the global object.
    pub fn get(&self, key: &str) -> Option<usize> {
        self.properties.get(key).copied()
    }

    /// Sets a property on the global object.
    pub fn set(&mut self, key: String, value: usize) {
        self.properties.insert(key, value);
    }

    /// Checks if a property exists on the global object.
    pub fn has(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Deletes a property from the global object.
    pub fn delete(&mut self, key: &str) -> bool {
        self.properties.remove(key).is_some()
    }

    /// Gets all property keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.properties.keys().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weakref() {
        let mut weakref = WeakRef::new(42);
        assert_eq!(weakref.deref(), Some(42));

        weakref.clear();
        assert_eq!(weakref.deref(), None);
    }

    #[test]
    fn test_finalization_registry() {
        let mut registry = FinalizationRegistry::new(1);

        // Register a target with held value and token
        registry.register(100, Some(200), Some(300));

        // Unregister using the token
        assert!(registry.unregister(300));
        assert!(!registry.unregister(300)); // Already unregistered
    }

    #[test]
    fn test_global_this() {
        let mut global = GlobalThis::new();

        global.set("foo".to_string(), 42);
        assert!(global.has("foo"));
        assert_eq!(global.get("foo"), Some(42));

        global.delete("foo");
        assert!(!global.has("foo"));
    }
}
