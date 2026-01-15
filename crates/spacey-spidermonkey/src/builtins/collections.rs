//! Built-in collection types: Map, Set, WeakMap, WeakSet.

use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// A JavaScript Map implementation.
#[derive(Debug, Clone)]
pub struct JsMap {
    /// The map entries (key-value pairs)
    entries: FxHashMap<MapKey, MapValue>,
    /// Insertion order for iteration
    insertion_order: Vec<MapKey>,
}

/// A key in a JavaScript Map.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    /// Undefined key
    Undefined,
    /// Null key
    Null,
    /// Boolean key
    Boolean(bool),
    /// Number key (as bits for hashing)
    Number(u64),
    /// String key
    String(String),
    /// Symbol key
    Symbol(u64),
    /// Object key (reference)
    Object(usize),
}

/// A value in a JavaScript Map.
#[derive(Debug, Clone, Default)]
pub enum MapValue {
    /// Undefined
    #[default]
    Undefined,
    /// Null
    Null,
    /// Boolean
    Boolean(bool),
    /// Number
    Number(f64),
    /// String
    String(String),
    /// Symbol
    Symbol(u64),
    /// BigInt
    BigInt(String),
    /// Object reference
    Object(usize),
}

impl JsMap {
    /// Creates a new empty Map.
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
            insertion_order: Vec::new(),
        }
    }

    /// Returns the number of entries in the map.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Gets a value by key.
    pub fn get(&self, key: &MapKey) -> Option<&MapValue> {
        self.entries.get(key)
    }

    /// Sets a value by key.
    pub fn set(&mut self, key: MapKey, value: MapValue) {
        if !self.entries.contains_key(&key) {
            self.insertion_order.push(key.clone());
        }
        self.entries.insert(key, value);
    }

    /// Checks if a key exists.
    pub fn has(&self, key: &MapKey) -> bool {
        self.entries.contains_key(key)
    }

    /// Deletes a key.
    pub fn delete(&mut self, key: &MapKey) -> bool {
        if self.entries.remove(key).is_some() {
            self.insertion_order.retain(|k| k != key);
            true
        } else {
            false
        }
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }

    /// Returns an iterator over entries in insertion order.
    pub fn entries(&self) -> impl Iterator<Item = (&MapKey, &MapValue)> {
        self.insertion_order
            .iter()
            .filter_map(|k| self.entries.get(k).map(|v| (k, v)))
    }

    /// Returns an iterator over keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &MapKey> {
        self.insertion_order.iter()
    }

    /// Returns an iterator over values in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &MapValue> {
        self.insertion_order
            .iter()
            .filter_map(|k| self.entries.get(k))
    }
}

impl Default for JsMap {
    fn default() -> Self {
        Self::new()
    }
}

/// A JavaScript Set implementation.
#[derive(Debug, Clone)]
pub struct JsSet {
    /// The set values
    values: HashSet<MapKey>,
    /// Insertion order for iteration
    insertion_order: Vec<MapKey>,
}

impl JsSet {
    /// Creates a new empty Set.
    pub fn new() -> Self {
        Self {
            values: HashSet::new(),
            insertion_order: Vec::new(),
        }
    }

    /// Returns the number of values in the set.
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Adds a value to the set.
    pub fn add(&mut self, value: MapKey) -> bool {
        if self.values.insert(value.clone()) {
            self.insertion_order.push(value);
            true
        } else {
            false
        }
    }

    /// Checks if a value exists.
    pub fn has(&self, value: &MapKey) -> bool {
        self.values.contains(value)
    }

    /// Deletes a value.
    pub fn delete(&mut self, value: &MapKey) -> bool {
        if self.values.remove(value) {
            self.insertion_order.retain(|v| v != value);
            true
        } else {
            false
        }
    }

    /// Clears all values.
    pub fn clear(&mut self) {
        self.values.clear();
        self.insertion_order.clear();
    }

    /// Returns an iterator over values in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &MapKey> {
        self.insertion_order.iter()
    }
}

impl Default for JsSet {
    fn default() -> Self {
        Self::new()
    }
}

/// A JavaScript WeakMap implementation.
/// Note: This is a simplified version without actual weak references.
/// A full implementation would need GC integration.
#[derive(Debug, Clone, Default)]
pub struct JsWeakMap {
    /// Object keys to values (only objects can be keys)
    entries: FxHashMap<usize, MapValue>,
}

impl JsWeakMap {
    /// Creates a new empty WeakMap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a value by object key.
    pub fn get(&self, key: usize) -> Option<&MapValue> {
        self.entries.get(&key)
    }

    /// Sets a value by object key.
    pub fn set(&mut self, key: usize, value: MapValue) {
        self.entries.insert(key, value);
    }

    /// Checks if an object key exists.
    pub fn has(&self, key: usize) -> bool {
        self.entries.contains_key(&key)
    }

    /// Deletes an object key.
    pub fn delete(&mut self, key: usize) -> bool {
        self.entries.remove(&key).is_some()
    }
}

/// A JavaScript WeakSet implementation.
/// Note: This is a simplified version without actual weak references.
#[derive(Debug, Clone, Default)]
pub struct JsWeakSet {
    /// Object values (only objects can be values)
    values: HashSet<usize>,
}

impl JsWeakSet {
    /// Creates a new empty WeakSet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an object value.
    pub fn add(&mut self, value: usize) -> bool {
        self.values.insert(value)
    }

    /// Checks if an object value exists.
    pub fn has(&self, value: usize) -> bool {
        self.values.contains(&value)
    }

    /// Deletes an object value.
    pub fn delete(&mut self, value: usize) -> bool {
        self.values.remove(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_operations() {
        let mut map = JsMap::new();
        assert_eq!(map.size(), 0);

        map.set(MapKey::String("key1".to_string()), MapValue::Number(42.0));
        assert_eq!(map.size(), 1);
        assert!(map.has(&MapKey::String("key1".to_string())));

        if let Some(MapValue::Number(n)) = map.get(&MapKey::String("key1".to_string())) {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number value");
        }

        map.delete(&MapKey::String("key1".to_string()));
        assert_eq!(map.size(), 0);
    }

    #[test]
    fn test_set_operations() {
        let mut set = JsSet::new();
        assert_eq!(set.size(), 0);

        set.add(MapKey::Number(1.0f64.to_bits()));
        set.add(MapKey::Number(2.0f64.to_bits()));
        set.add(MapKey::Number(1.0f64.to_bits())); // Duplicate

        assert_eq!(set.size(), 2);
        assert!(set.has(&MapKey::Number(1.0f64.to_bits())));

        set.delete(&MapKey::Number(1.0f64.to_bits()));
        assert_eq!(set.size(), 1);
    }
}
