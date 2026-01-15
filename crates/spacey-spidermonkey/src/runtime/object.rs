//! JavaScript object representation.

use super::value::Value;
use rustc_hash::FxHashMap;

/// A JavaScript object.
#[derive(Debug, Clone)]
pub struct Object {
    /// The prototype of this object
    pub prototype: Option<Box<Object>>,
    /// The properties
    pub properties: FxHashMap<String, Property>,
    /// Whether the object is extensible
    pub extensible: bool,
}

impl Object {
    /// Creates a new empty object.
    pub fn new() -> Self {
        Self {
            prototype: None,
            properties: FxHashMap::default(),
            extensible: true,
        }
    }

    /// Gets a property value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.properties.get(key).map(|p| &p.value)
    }

    /// Sets a property value.
    pub fn set(&mut self, key: String, value: Value) {
        self.properties.insert(
            key,
            Property {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            },
        );
    }

    /// Deletes a property.
    pub fn delete(&mut self, key: &str) -> bool {
        if let Some(prop) = self.properties.get(key)
            && prop.configurable
        {
            self.properties.remove(key);
            return true;
        }
        false
    }

    /// Checks if a property exists.
    pub fn has(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

/// A property descriptor.
#[derive(Debug, Clone)]
pub struct Property {
    /// The property value
    pub value: Value,
    /// Whether the property is writable
    pub writable: bool,
    /// Whether the property is enumerable
    pub enumerable: bool,
    /// Whether the property is configurable
    pub configurable: bool,
}
