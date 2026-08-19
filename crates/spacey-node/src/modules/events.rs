// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `events` module - EventEmitter implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the events module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // EventEmitter class would be registered here
    // For now, return module structure
    exports.insert("defaultMaxListeners".to_string(), Value::Number(10.0));

    Value::NativeObject(exports)
}

/// EventEmitter implementation
#[derive(Debug, Default)]
pub struct EventEmitter {
    /// Event listeners
    listeners: HashMap<String, Vec<Listener>>,
    /// Maximum listeners per event
    max_listeners: usize,
}

/// A listener entry
#[derive(Debug, Clone)]
struct Listener {
    /// The callback function
    callback: Value,
    /// Whether this is a one-time listener
    once: bool,
}

impl EventEmitter {
    /// Create a new EventEmitter
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            max_listeners: 10,
        }
    }

    /// Add an event listener
    pub fn on(&mut self, event: &str, callback: Value) -> &mut Self {
        self.add_listener(event, callback, false)
    }

    /// Add a one-time event listener
    pub fn once(&mut self, event: &str, callback: Value) -> &mut Self {
        self.add_listener(event, callback, true)
    }

    /// Add a listener (internal)
    fn add_listener(&mut self, event: &str, callback: Value, once: bool) -> &mut Self {
        let listeners = self.listeners.entry(event.to_string()).or_default();

        if listeners.len() >= self.max_listeners && self.max_listeners > 0 {
            eprintln!(
                "Warning: Possible EventEmitter memory leak detected. {} listeners added to event '{}'. Use emitter.setMaxListeners() to increase limit.",
                listeners.len() + 1,
                event
            );
        }

        listeners.push(Listener { callback, once });
        self
    }

    /// Add a listener to the beginning of the listeners array
    pub fn prepend_listener(&mut self, event: &str, callback: Value) -> &mut Self {
        let listeners = self.listeners.entry(event.to_string()).or_default();
        listeners.insert(
            0,
            Listener {
                callback,
                once: false,
            },
        );
        self
    }

    /// Add a one-time listener to the beginning
    pub fn prepend_once_listener(&mut self, event: &str, callback: Value) -> &mut Self {
        let listeners = self.listeners.entry(event.to_string()).or_default();
        listeners.insert(
            0,
            Listener {
                callback,
                once: true,
            },
        );
        self
    }

    /// Remove a listener
    pub fn remove_listener(&mut self, event: &str, callback: &Value) -> &mut Self {
        if let Some(listeners) = self.listeners.get_mut(event) {
            // Find and remove the first matching listener
            if let Some(idx) = listeners.iter().position(|l| &l.callback == callback) {
                listeners.remove(idx);
            }
        }
        self
    }

    /// Alias for remove_listener
    pub fn off(&mut self, event: &str, callback: &Value) -> &mut Self {
        self.remove_listener(event, callback)
    }

    /// Remove all listeners for an event
    pub fn remove_all_listeners(&mut self, event: Option<&str>) -> &mut Self {
        match event {
            Some(e) => {
                self.listeners.remove(e);
            }
            None => {
                self.listeners.clear();
            }
        }
        self
    }

    /// Emit an event
    pub fn emit(&mut self, event: &str, args: &[Value]) -> bool {
        let listeners = match self.listeners.get_mut(event) {
            Some(l) if !l.is_empty() => l,
            _ => return false,
        };

        // Collect listeners to call (and track which to remove)
        let to_call: Vec<_> = listeners.clone();

        // Remove once listeners
        listeners.retain(|l| !l.once);

        // Call listeners
        for listener in to_call {
            // In a real implementation, we'd call the function through the engine
            // For now, we just note that the listener was found
            tracing::debug!("Emitting {} with callback {:?}", event, listener.callback);
        }

        true
    }

    /// Get listener count for an event
    pub fn listener_count(&self, event: &str) -> usize {
        self.listeners.get(event).map(|l| l.len()).unwrap_or(0)
    }

    /// Get listeners for an event
    pub fn listeners(&self, event: &str) -> Vec<Value> {
        self.listeners
            .get(event)
            .map(|l| l.iter().map(|listener| listener.callback.clone()).collect())
            .unwrap_or_default()
    }

    /// Get raw listeners (including wrapper for once)
    pub fn raw_listeners(&self, event: &str) -> Vec<Value> {
        self.listeners(event)
    }

    /// Get all event names
    pub fn event_names(&self) -> Vec<String> {
        self.listeners.keys().cloned().collect()
    }

    /// Get/set max listeners
    pub fn get_max_listeners(&self) -> usize {
        self.max_listeners
    }

    pub fn set_max_listeners(&mut self, n: usize) -> &mut Self {
        self.max_listeners = n;
        self
    }
}

/// Static method: get default max listeners
pub fn default_max_listeners() -> usize {
    10
}

/// Static method: set default max listeners
pub fn set_default_max_listeners(_n: usize) {
    // Would set a global default
}

/// events.once() - create a promise that resolves on first event
pub async fn once(_emitter: &EventEmitter, _event: &str) -> Vec<Value> {
    // Would return a promise that resolves when the event fires
    vec![]
}

/// events.on() - create an async iterator for events
pub fn on(_emitter: &EventEmitter, _event: &str) {
    // Would return an async iterator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emitter_on() {
        let mut emitter = EventEmitter::new();
        let callback = Value::String("callback".to_string());

        emitter.on("test", callback.clone());
        assert_eq!(emitter.listener_count("test"), 1);
    }

    #[test]
    fn test_event_emitter_remove() {
        let mut emitter = EventEmitter::new();
        let callback = Value::String("callback".to_string());

        emitter.on("test", callback.clone());
        assert_eq!(emitter.listener_count("test"), 1);

        emitter.remove_listener("test", &callback);
        assert_eq!(emitter.listener_count("test"), 0);
    }

    #[test]
    fn test_event_emitter_once() {
        let mut emitter = EventEmitter::new();
        let callback = Value::String("callback".to_string());

        emitter.once("test", callback);
        assert_eq!(emitter.listener_count("test"), 1);

        emitter.emit("test", &[]);
        assert_eq!(emitter.listener_count("test"), 0);
    }

    #[test]
    fn test_event_names() {
        let mut emitter = EventEmitter::new();
        emitter.on("event1", Value::Null);
        emitter.on("event2", Value::Null);

        let names = emitter.event_names();
        assert!(names.contains(&"event1".to_string()));
        assert!(names.contains(&"event2".to_string()));
    }
}
