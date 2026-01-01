//! JavaScript object representation.
//!
//! This module implements JavaScript objects with ES3-compliant prototype chain
//! support, property descriptors, and internal methods.

use super::value::Value;
use rustc_hash::FxHashMap;
use std::sync::Arc;

// ============================================================================
// Object Kind - Distinguishes different object types
// ============================================================================

/// The internal [[Class]] of an object (ES3 Section 8.6.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// Ordinary object
    Object,
    /// Array object
    Array,
    /// Function object
    Function,
    /// String wrapper object
    String,
    /// Number wrapper object
    Number,
    /// Boolean wrapper object
    Boolean,
    /// Date object
    Date,
    /// RegExp object
    RegExp,
    /// Error object
    Error,
    /// Arguments object
    Arguments,
    /// Math object (singleton)
    Math,
    /// JSON object (singleton)
    Json,
}

impl ObjectKind {
    /// Returns the [[Class]] name for this object kind
    pub fn class_name(&self) -> &'static str {
        match self {
            ObjectKind::Object => "Object",
            ObjectKind::Array => "Array",
            ObjectKind::Function => "Function",
            ObjectKind::String => "String",
            ObjectKind::Number => "Number",
            ObjectKind::Boolean => "Boolean",
            ObjectKind::Date => "Date",
            ObjectKind::RegExp => "RegExp",
            ObjectKind::Error => "Error",
            ObjectKind::Arguments => "Arguments",
            ObjectKind::Math => "Math",
            ObjectKind::Json => "JSON",
        }
    }
}

// ============================================================================
// Property Descriptor (ES3 Section 8.6.1)
// ============================================================================

/// A property descriptor.
#[derive(Debug, Clone)]
pub struct Property {
    /// The property value
    pub value: Value,
    /// Whether the property is writable (ES3: [[Writable]])
    pub writable: bool,
    /// Whether the property is enumerable (ES3: [[Enumerable]])
    pub enumerable: bool,
    /// Whether the property is configurable (ES3: [[Configurable]])
    pub configurable: bool,
}

impl Property {
    /// Creates a new data property with default attributes.
    pub fn new(value: Value) -> Self {
        Self {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        }
    }

    /// Creates a read-only property.
    pub fn readonly(value: Value) -> Self {
        Self {
            value,
            writable: false,
            enumerable: true,
            configurable: true,
        }
    }

    /// Creates a non-enumerable property.
    pub fn hidden(value: Value) -> Self {
        Self {
            value,
            writable: true,
            enumerable: false,
            configurable: true,
        }
    }

    /// Creates a non-configurable property.
    pub fn sealed(value: Value) -> Self {
        Self {
            value,
            writable: true,
            enumerable: true,
            configurable: false,
        }
    }

    /// Creates a property with all attributes set to false (frozen).
    pub fn frozen(value: Value) -> Self {
        Self {
            value,
            writable: false,
            enumerable: true,
            configurable: false,
        }
    }
}

impl Default for Property {
    fn default() -> Self {
        Self::new(Value::Undefined)
    }
}

// ============================================================================
// Object (ES3 Section 8.6)
// ============================================================================

/// A JavaScript object.
#[derive(Debug, Clone)]
pub struct Object {
    /// The [[Class]] internal property
    pub kind: ObjectKind,
    /// The [[Prototype]] internal property
    pub prototype: Option<Arc<Object>>,
    /// The named properties
    pub properties: FxHashMap<String, Property>,
    /// Whether the object is extensible
    pub extensible: bool,
    /// Primitive value for wrapper objects (String, Number, Boolean)
    pub primitive_value: Option<Value>,
}

impl Object {
    /// Creates a new empty object with Object.prototype.
    pub fn new() -> Self {
        Self {
            kind: ObjectKind::Object,
            prototype: None, // Would be Object.prototype in full impl
            properties: FxHashMap::default(),
            extensible: true,
            primitive_value: None,
        }
    }

    /// Creates an object with a specific kind.
    pub fn with_kind(kind: ObjectKind) -> Self {
        Self {
            kind,
            prototype: None,
            properties: FxHashMap::default(),
            extensible: true,
            primitive_value: None,
        }
    }

    /// Creates an object with a specific prototype.
    pub fn with_prototype(prototype: Option<Arc<Object>>) -> Self {
        Self {
            kind: ObjectKind::Object,
            prototype,
            properties: FxHashMap::default(),
            extensible: true,
            primitive_value: None,
        }
    }

    /// Creates an array object.
    pub fn array() -> Self {
        let mut obj = Self::with_kind(ObjectKind::Array);
        obj.define_property("length", Property::new(Value::Number(0.0)));
        obj
    }

    /// Creates a string wrapper object.
    pub fn string(value: String) -> Self {
        let len = value.chars().count();
        let mut obj = Self::with_kind(ObjectKind::String);
        obj.primitive_value = Some(Value::String(value));
        obj.define_property("length", Property::readonly(Value::Number(len as f64)));
        obj
    }

    /// Creates a number wrapper object.
    pub fn number(value: f64) -> Self {
        let mut obj = Self::with_kind(ObjectKind::Number);
        obj.primitive_value = Some(Value::Number(value));
        obj
    }

    /// Creates a boolean wrapper object.
    pub fn boolean(value: bool) -> Self {
        let mut obj = Self::with_kind(ObjectKind::Boolean);
        obj.primitive_value = Some(Value::Boolean(value));
        obj
    }

    // ========================================================================
    // Property Access (ES3 Section 8.6.2)
    // ========================================================================

    /// [[Get]] - Gets a property value, walking the prototype chain.
    ///
    /// ES3 Section 8.6.2.1
    pub fn get(&self, key: &str) -> Option<Value> {
        // First check own properties
        if let Some(prop) = self.properties.get(key) {
            return Some(prop.value.clone());
        }

        // Walk the prototype chain
        if let Some(proto) = &self.prototype {
            return proto.get(key);
        }

        None
    }

    /// Gets an own property value (does not walk prototype chain).
    pub fn get_own(&self, key: &str) -> Option<&Value> {
        self.properties.get(key).map(|p| &p.value)
    }

    /// Gets a property descriptor.
    pub fn get_own_property(&self, key: &str) -> Option<&Property> {
        self.properties.get(key)
    }

    /// [[Put]] - Sets a property value.
    ///
    /// ES3 Section 8.6.2.2
    pub fn put(&mut self, key: String, value: Value) -> bool {
        // Check if property exists and is writable
        if let Some(prop) = self.properties.get(&key) {
            if !prop.writable {
                return false; // Cannot write to non-writable property
            }
        } else if !self.extensible {
            return false; // Cannot add new properties
        }

        // Set the property
        self.properties.insert(key, Property::new(value));
        true
    }

    /// Sets a property value (alias for put with simpler interface).
    pub fn set(&mut self, key: String, value: Value) {
        self.put(key, value);
    }

    /// [[CanPut]] - Checks if a property can be set.
    ///
    /// ES3 Section 8.6.2.3
    pub fn can_put(&self, key: &str) -> bool {
        // Check own property
        if let Some(prop) = self.properties.get(key) {
            return prop.writable;
        }

        // Check prototype chain for inherited property
        if let Some(proto) = &self.prototype
            && let Some(prop) = proto.get_own_property(key) {
                return prop.writable;
            }

        // Property doesn't exist, check extensibility
        self.extensible
    }

    /// [[HasProperty]] - Checks if a property exists (including prototype chain).
    ///
    /// ES3 Section 8.6.2.4
    pub fn has_property(&self, key: &str) -> bool {
        if self.properties.contains_key(key) {
            return true;
        }

        if let Some(proto) = &self.prototype {
            return proto.has_property(key);
        }

        false
    }

    /// Checks if an own property exists.
    pub fn has_own_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Alias for has_own_property (legacy compatibility).
    pub fn has(&self, key: &str) -> bool {
        self.has_own_property(key)
    }

    /// [[Delete]] - Deletes a property.
    ///
    /// ES3 Section 8.6.2.5
    pub fn delete(&mut self, key: &str) -> bool {
        if let Some(prop) = self.properties.get(key)
            && !prop.configurable {
                return false; // Cannot delete non-configurable property
            }

        self.properties.remove(key);
        true
    }

    /// [[DefaultValue]] - Gets the default primitive value.
    ///
    /// ES3 Section 8.6.2.6
    pub fn default_value(&self, _hint: Option<&str>) -> Value {
        // For wrapper objects, return the primitive value
        if let Some(ref prim) = self.primitive_value {
            return prim.clone();
        }

        // Default to string representation
        Value::String(format!("[object {}]", self.kind.class_name()))
    }

    // ========================================================================
    // Property Definition
    // ========================================================================

    /// Defines a property with full descriptor control.
    pub fn define_property(&mut self, key: &str, property: Property) {
        self.properties.insert(key.to_string(), property);
    }

    /// Defines multiple properties at once.
    pub fn define_properties(&mut self, properties: &[(&str, Property)]) {
        for (key, prop) in properties {
            self.properties.insert((*key).to_string(), prop.clone());
        }
    }

    // ========================================================================
    // Object State
    // ========================================================================

    /// Prevents new properties from being added.
    pub fn prevent_extensions(&mut self) {
        self.extensible = false;
    }

    /// Seals the object (prevents adding/removing properties).
    pub fn seal(&mut self) {
        self.extensible = false;
        for prop in self.properties.values_mut() {
            prop.configurable = false;
        }
    }

    /// Freezes the object (seals and makes all properties read-only).
    pub fn freeze(&mut self) {
        self.extensible = false;
        for prop in self.properties.values_mut() {
            prop.configurable = false;
            prop.writable = false;
        }
    }

    /// Checks if the object is sealed.
    pub fn is_sealed(&self) -> bool {
        if self.extensible {
            return false;
        }
        self.properties.values().all(|p| !p.configurable)
    }

    /// Checks if the object is frozen.
    pub fn is_frozen(&self) -> bool {
        if self.extensible {
            return false;
        }
        self.properties
            .values()
            .all(|p| !p.configurable && !p.writable)
    }

    // ========================================================================
    // Enumeration
    // ========================================================================

    /// Returns an iterator over own enumerable property names.
    pub fn own_enumerable_keys(&self) -> impl Iterator<Item = &String> {
        self.properties
            .iter()
            .filter(|(_, p)| p.enumerable)
            .map(|(k, _)| k)
    }

    /// Returns all own property names (enumerable and non-enumerable).
    pub fn own_property_names(&self) -> impl Iterator<Item = &String> {
        self.properties.keys()
    }

    /// Returns an iterator over own enumerable property entries.
    pub fn own_enumerable_entries(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.properties
            .iter()
            .filter(|(_, p)| p.enumerable)
            .map(|(k, p)| (k, &p.value))
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Basic Object Tests
    // ========================================================================

    #[test]
    fn test_new_object() {
        let obj = Object::new();
        assert!(obj.extensible);
        assert!(obj.prototype.is_none());
        assert!(!obj.has("x"));
        assert_eq!(obj.kind, ObjectKind::Object);
    }

    #[test]
    fn test_default_object() {
        let obj = Object::default();
        assert!(obj.extensible);
        assert!(obj.prototype.is_none());
    }

    #[test]
    fn test_set_and_get() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));

        assert!(obj.has("x"));
        assert_eq!(obj.get("x"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_get_nonexistent() {
        let obj = Object::new();
        assert!(obj.get("x").is_none());
    }

    #[test]
    fn test_overwrite_property() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(1.0));
        obj.set("x".to_string(), Value::Number(2.0));

        assert_eq!(obj.get("x"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_delete_configurable() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));
        assert!(obj.has("x"));

        assert!(obj.delete("x"));
        assert!(!obj.has("x"));
    }

    #[test]
    fn test_delete_non_configurable() {
        let mut obj = Object::new();
        obj.define_property("x", Property::sealed(Value::Number(42.0)));

        assert!(!obj.delete("x"));
        assert!(obj.has("x"));
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut obj = Object::new();
        // Deleting nonexistent property returns true (per ES3)
        assert!(obj.delete("x"));
    }

    #[test]
    fn test_has() {
        let mut obj = Object::new();
        assert!(!obj.has("x"));
        obj.set("x".to_string(), Value::Undefined);
        assert!(obj.has("x"));
    }

    #[test]
    fn test_multiple_properties() {
        let mut obj = Object::new();
        obj.set("a".to_string(), Value::Number(1.0));
        obj.set("b".to_string(), Value::String("hello".to_string()));
        obj.set("c".to_string(), Value::Boolean(true));
        obj.set("d".to_string(), Value::Null);

        assert_eq!(obj.get("a"), Some(Value::Number(1.0)));
        assert_eq!(obj.get("b"), Some(Value::String("hello".to_string())));
        assert_eq!(obj.get("c"), Some(Value::Boolean(true)));
        assert_eq!(obj.get("d"), Some(Value::Null));
    }

    // ========================================================================
    // Property Descriptor Tests
    // ========================================================================

    #[test]
    fn test_property_new() {
        let prop = Property::new(Value::Number(42.0));
        assert!(prop.writable);
        assert!(prop.enumerable);
        assert!(prop.configurable);
    }

    #[test]
    fn test_property_readonly() {
        let prop = Property::readonly(Value::Number(42.0));
        assert!(!prop.writable);
        assert!(prop.enumerable);
        assert!(prop.configurable);
    }

    #[test]
    fn test_property_hidden() {
        let prop = Property::hidden(Value::Number(42.0));
        assert!(prop.writable);
        assert!(!prop.enumerable);
        assert!(prop.configurable);
    }

    #[test]
    fn test_property_sealed() {
        let prop = Property::sealed(Value::Number(42.0));
        assert!(prop.writable);
        assert!(prop.enumerable);
        assert!(!prop.configurable);
    }

    #[test]
    fn test_property_frozen() {
        let prop = Property::frozen(Value::Number(42.0));
        assert!(!prop.writable);
        assert!(prop.enumerable);
        assert!(!prop.configurable);
    }

    #[test]
    fn test_property_default() {
        let prop = Property::default();
        assert_eq!(prop.value, Value::Undefined);
    }

    #[test]
    fn test_property_descriptor_defaults() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));

        let prop = obj.get_own_property("x").unwrap();
        assert!(prop.writable);
        assert!(prop.enumerable);
        assert!(prop.configurable);
    }

    // ========================================================================
    // Prototype Chain Tests
    // ========================================================================

    #[test]
    fn test_prototype_get() {
        let mut proto = Object::new();
        proto.set("inherited".to_string(), Value::Number(100.0));

        let obj = Object::with_prototype(Some(Arc::new(proto)));

        // Should find inherited property
        assert_eq!(obj.get("inherited"), Some(Value::Number(100.0)));
    }

    #[test]
    fn test_prototype_own_shadows_inherited() {
        let mut proto = Object::new();
        proto.set("x".to_string(), Value::Number(1.0));

        let mut obj = Object::with_prototype(Some(Arc::new(proto)));
        obj.set("x".to_string(), Value::Number(2.0));

        // Own property shadows inherited
        assert_eq!(obj.get("x"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_prototype_has_property() {
        let mut proto = Object::new();
        proto.set("inherited".to_string(), Value::Number(100.0));

        let obj = Object::with_prototype(Some(Arc::new(proto)));

        assert!(obj.has_property("inherited"));
        assert!(!obj.has_own_property("inherited"));
    }

    #[test]
    fn test_get_own_vs_get() {
        let mut proto = Object::new();
        proto.set("inherited".to_string(), Value::Number(100.0));

        let obj = Object::with_prototype(Some(Arc::new(proto)));

        // get() walks prototype chain
        assert_eq!(obj.get("inherited"), Some(Value::Number(100.0)));
        // get_own() doesn't
        assert!(obj.get_own("inherited").is_none());
    }

    // ========================================================================
    // Object Kind Tests
    // ========================================================================

    #[test]
    fn test_object_kind_class_name() {
        assert_eq!(ObjectKind::Object.class_name(), "Object");
        assert_eq!(ObjectKind::Array.class_name(), "Array");
        assert_eq!(ObjectKind::Function.class_name(), "Function");
        assert_eq!(ObjectKind::String.class_name(), "String");
        assert_eq!(ObjectKind::Number.class_name(), "Number");
        assert_eq!(ObjectKind::Boolean.class_name(), "Boolean");
        assert_eq!(ObjectKind::Date.class_name(), "Date");
        assert_eq!(ObjectKind::RegExp.class_name(), "RegExp");
        assert_eq!(ObjectKind::Error.class_name(), "Error");
        assert_eq!(ObjectKind::Arguments.class_name(), "Arguments");
        assert_eq!(ObjectKind::Math.class_name(), "Math");
        assert_eq!(ObjectKind::Json.class_name(), "JSON");
    }

    #[test]
    fn test_object_with_kind() {
        let obj = Object::with_kind(ObjectKind::Array);
        assert_eq!(obj.kind, ObjectKind::Array);
    }

    // ========================================================================
    // Wrapper Object Tests
    // ========================================================================

    #[test]
    fn test_array_object() {
        let arr = Object::array();
        assert_eq!(arr.kind, ObjectKind::Array);
        assert_eq!(arr.get("length"), Some(Value::Number(0.0)));
    }

    #[test]
    fn test_string_wrapper() {
        let obj = Object::string("hello".to_string());
        assert_eq!(obj.kind, ObjectKind::String);
        assert_eq!(obj.get("length"), Some(Value::Number(5.0)));
        assert_eq!(
            obj.primitive_value,
            Some(Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_number_wrapper() {
        let obj = Object::number(42.0);
        assert_eq!(obj.kind, ObjectKind::Number);
        assert_eq!(obj.primitive_value, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_boolean_wrapper() {
        let obj = Object::boolean(true);
        assert_eq!(obj.kind, ObjectKind::Boolean);
        assert_eq!(obj.primitive_value, Some(Value::Boolean(true)));
    }

    #[test]
    fn test_default_value_wrapper() {
        let obj = Object::number(42.0);
        assert_eq!(obj.default_value(None), Value::Number(42.0));
    }

    #[test]
    fn test_default_value_object() {
        let obj = Object::new();
        assert_eq!(
            obj.default_value(None),
            Value::String("[object Object]".to_string())
        );
    }

    // ========================================================================
    // Object State Tests (freeze, seal, prevent extensions)
    // ========================================================================

    #[test]
    fn test_prevent_extensions() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(1.0));

        obj.prevent_extensions();
        assert!(!obj.extensible);

        // put() should fail for new properties
        assert!(!obj.put("y".to_string(), Value::Number(2.0)));
        assert!(!obj.has("y"));

        // But existing properties can still be modified
        assert!(obj.put("x".to_string(), Value::Number(10.0)));
    }

    #[test]
    fn test_seal() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(1.0));

        obj.seal();
        assert!(!obj.extensible);
        assert!(obj.is_sealed());

        // Cannot add new properties
        obj.put("y".to_string(), Value::Number(2.0));
        assert!(!obj.has("y"));

        // Cannot delete properties
        assert!(!obj.delete("x"));

        // But can modify existing writable properties
        assert!(obj.put("x".to_string(), Value::Number(10.0)));
    }

    #[test]
    fn test_freeze() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(1.0));

        obj.freeze();
        assert!(!obj.extensible);
        assert!(obj.is_frozen());

        // Cannot add properties
        obj.put("y".to_string(), Value::Number(2.0));
        assert!(!obj.has("y"));

        // Cannot delete properties
        assert!(!obj.delete("x"));

        // Cannot modify properties
        assert!(!obj.put("x".to_string(), Value::Number(10.0)));
    }

    #[test]
    fn test_can_put() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(1.0));

        assert!(obj.can_put("x"));
        assert!(obj.can_put("y")); // Can add new properties

        obj.prevent_extensions();
        assert!(obj.can_put("x")); // Can still modify existing
        assert!(!obj.can_put("y")); // Cannot add new
    }

    // ========================================================================
    // Enumeration Tests
    // ========================================================================

    #[test]
    fn test_own_enumerable_keys() {
        let mut obj = Object::new();
        obj.set("a".to_string(), Value::Number(1.0));
        obj.define_property("b", Property::hidden(Value::Number(2.0)));
        obj.set("c".to_string(), Value::Number(3.0));

        let keys: Vec<_> = obj.own_enumerable_keys().collect();
        assert!(keys.contains(&&"a".to_string()));
        assert!(!keys.contains(&&"b".to_string())); // Hidden
        assert!(keys.contains(&&"c".to_string()));
    }

    #[test]
    fn test_own_property_names() {
        let mut obj = Object::new();
        obj.set("a".to_string(), Value::Number(1.0));
        obj.define_property("b", Property::hidden(Value::Number(2.0)));

        let names: Vec<_> = obj.own_property_names().collect();
        assert!(names.contains(&&"a".to_string()));
        assert!(names.contains(&&"b".to_string())); // Includes non-enumerable
    }

    // ========================================================================
    // Clone and Debug Tests
    // ========================================================================

    #[test]
    fn test_object_clone() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));

        let cloned = obj.clone();
        assert_eq!(cloned.get("x"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_property_debug() {
        let prop = Property::new(Value::Number(42.0));
        let debug_str = format!("{:?}", prop);
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_object_debug() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("properties"));
    }

    #[test]
    fn test_define_properties() {
        let mut obj = Object::new();
        obj.define_properties(&[
            ("a", Property::new(Value::Number(1.0))),
            ("b", Property::readonly(Value::Number(2.0))),
        ]);

        assert_eq!(obj.get("a"), Some(Value::Number(1.0)));
        assert_eq!(obj.get("b"), Some(Value::Number(2.0)));
        assert!(!obj.get_own_property("b").unwrap().writable);
    }
}
