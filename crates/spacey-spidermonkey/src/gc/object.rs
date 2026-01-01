//! GC object representation with compact headers.
//!
//! Objects are designed for cache efficiency and fast GC operations.

use std::sync::atomic::{AtomicU8, Ordering};

use rustc_hash::FxHashMap;

/// Compact object header for GC metadata.
///
/// Layout is carefully designed to fit in a single cache line
/// along with small objects.
#[repr(C)]
pub struct ObjectHeader {
    /// Tri-color mark state (White=0, Gray=1, Black=2)
    pub color: AtomicU8,
    /// Object age for generational GC (0-255)
    pub age: AtomicU8,
    /// Object flags (frozen, sealed, extensible, etc.)
    pub flags: AtomicU8,
    /// Reserved for future use
    _reserved: u8,
}

impl ObjectHeader {
    /// Creates a new object header.
    pub fn new() -> Self {
        Self {
            color: AtomicU8::new(0), // White
            age: AtomicU8::new(0),
            flags: AtomicU8::new(ObjectFlags::EXTENSIBLE),
            _reserved: 0,
        }
    }

    /// Increments the age, saturating at 255.
    #[inline]
    pub fn increment_age(&self) {
        let current = self.age.load(Ordering::Relaxed);
        if current < 255 {
            self.age.store(current + 1, Ordering::Relaxed);
        }
    }

    /// Checks if the object is frozen.
    #[inline]
    pub fn is_frozen(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & ObjectFlags::FROZEN != 0
    }

    /// Checks if the object is sealed.
    #[inline]
    pub fn is_sealed(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & ObjectFlags::SEALED != 0
    }

    /// Checks if the object is extensible.
    #[inline]
    pub fn is_extensible(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & ObjectFlags::EXTENSIBLE != 0
    }
}

impl Default for ObjectHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Object flags stored in the header.
pub struct ObjectFlags;

#[allow(dead_code)]
impl ObjectFlags {
    /// Object is extensible (can add properties)
    pub const EXTENSIBLE: u8 = 0b0000_0001;
    /// Object is sealed (cannot add/delete properties)
    pub const SEALED: u8 = 0b0000_0010;
    /// Object is frozen (cannot modify properties)
    pub const FROZEN: u8 = 0b0000_0100;
    /// Object is a prototype
    pub const PROTOTYPE: u8 = 0b0000_1000;
    /// Object has been finalized
    pub const FINALIZED: u8 = 0b0001_0000;
}

/// A reference to a garbage-collected object.
///
/// The reference encodes whether the object is in the young or old generation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcRef {
    /// Packed value: bit 63 = generation (0=young, 1=old), bits 0-62 = index
    value: u64,
}

impl GcRef {
    const GENERATION_BIT: u64 = 1 << 63;
    const INDEX_MASK: u64 = !Self::GENERATION_BIT;

    /// Creates a reference to a young generation object.
    #[inline]
    pub fn new_young(index: usize) -> Self {
        Self {
            value: index as u64,
        }
    }

    /// Creates a reference to an old generation object.
    #[inline]
    pub fn new_old(index: usize) -> Self {
        Self {
            value: (index as u64) | Self::GENERATION_BIT,
        }
    }

    /// Returns the index of this reference.
    #[inline]
    pub fn index(&self) -> usize {
        (self.value & Self::INDEX_MASK) as usize
    }

    /// Returns whether this reference points to a young generation object.
    #[inline]
    pub fn is_young(&self) -> bool {
        self.value & Self::GENERATION_BIT == 0
    }

    /// Returns whether this reference points to an old generation object.
    #[inline]
    pub fn is_old(&self) -> bool {
        !self.is_young()
    }

    /// Creates a null reference (index 0 in young gen).
    #[inline]
    pub fn null() -> Self {
        Self { value: 0 }
    }
}

impl std::fmt::Debug for GcRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_young() {
            write!(f, "GcRef(young:{})", self.index())
        } else {
            write!(f, "GcRef(old:{})", self.index())
        }
    }
}

/// A garbage-collected object with header.
pub struct GcObject {
    header: ObjectHeader,
    data: JsObject,
}

impl GcObject {
    /// Creates a new GC object.
    pub fn new(data: JsObject) -> Self {
        Self {
            header: ObjectHeader::new(),
            data,
        }
    }

    /// Returns a reference to the header.
    #[inline]
    pub fn header(&self) -> &ObjectHeader {
        &self.header
    }

    /// Returns a reference to the object data.
    #[inline]
    pub fn data(&self) -> &JsObject {
        &self.data
    }

    /// Returns a mutable reference to the object data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut JsObject {
        &mut self.data
    }

    /// Returns the size of this object in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.size_bytes()
    }
}

/// A property value that can reference other objects.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number value (IEEE 754 double)
    Number(f64),
    /// String value (interned for common strings)
    String(String),
    /// Object reference
    Object(GcRef),
    /// Symbol reference
    Symbol(u64),
}

impl PropertyValue {
    /// Returns the approximate size in bytes.
    #[inline]
    pub fn size_bytes(&self) -> usize {
        match self {
            PropertyValue::String(s) => std::mem::size_of::<String>() + s.len(),
            _ => std::mem::size_of::<Self>(),
        }
    }
}

/// A JavaScript object stored on the heap.
///
/// Uses FxHashMap for faster property access compared to std HashMap.
#[derive(Debug, Clone)]
pub struct JsObject {
    /// The prototype reference (if any)
    pub prototype: Option<GcRef>,
    /// Object properties (using FxHashMap for speed)
    pub properties: FxHashMap<String, PropertyValue>,
    /// Hidden class/shape for inline caching (future optimization)
    #[allow(dead_code)]
    shape_id: u32,
}

impl JsObject {
    /// Creates a new empty object.
    #[inline]
    pub fn new() -> Self {
        Self {
            prototype: None,
            properties: FxHashMap::default(),
            shape_id: 0,
        }
    }

    /// Creates a new object with a prototype.
    #[inline]
    pub fn with_prototype(prototype: GcRef) -> Self {
        Self {
            prototype: Some(prototype),
            properties: FxHashMap::default(),
            shape_id: 0,
        }
    }

    /// Creates a new object with initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            prototype: None,
            properties: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            shape_id: 0,
        }
    }

    /// Gets a property value.
    #[inline]
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Sets a property value.
    #[inline]
    pub fn set(&mut self, name: String, value: PropertyValue) {
        self.properties.insert(name, value);
    }

    /// Deletes a property.
    #[inline]
    pub fn delete(&mut self, name: &str) -> bool {
        self.properties.remove(name).is_some()
    }

    /// Returns the number of properties.
    #[inline]
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Returns whether the object has no properties.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Returns the approximate size in bytes.
    pub fn size_bytes(&self) -> usize {
        let base_size = std::mem::size_of::<Self>();
        let properties_size: usize = self
            .properties
            .iter()
            .map(|(k, v)| k.len() + v.size_bytes())
            .sum();
        base_size + properties_size
    }

    /// Returns all GC references held by this object.
    pub fn trace_refs(&self) -> Vec<GcRef> {
        let mut refs = Vec::with_capacity(self.properties.len() + 1);

        if let Some(proto) = self.prototype {
            refs.push(proto);
        }

        for value in self.properties.values() {
            if let PropertyValue::Object(gc_ref) = value {
                refs.push(*gc_ref);
            }
        }

        refs
    }
}

impl Default for JsObject {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be garbage collected.
#[allow(dead_code)]
pub trait GcTrace: Send + Sync {
    /// Returns all GC references held by this object.
    fn trace_refs(&self) -> Vec<GcRef>;
}

#[allow(dead_code)]
impl GcTrace for JsObject {
    fn trace_refs(&self) -> Vec<GcRef> {
        self.trace_refs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_ref_young() {
        let gc_ref = GcRef::new_young(42);
        assert!(gc_ref.is_young());
        assert!(!gc_ref.is_old());
        assert_eq!(gc_ref.index(), 42);
    }

    #[test]
    fn test_gc_ref_old() {
        let gc_ref = GcRef::new_old(123);
        assert!(gc_ref.is_old());
        assert!(!gc_ref.is_young());
        assert_eq!(gc_ref.index(), 123);
    }

    #[test]
    fn test_object_header() {
        let header = ObjectHeader::new();
        assert!(header.is_extensible());
        assert!(!header.is_frozen());
        assert!(!header.is_sealed());
        assert_eq!(header.age.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_js_object_properties() {
        let mut obj = JsObject::new();
        obj.set("x".to_string(), PropertyValue::Number(42.0));
        obj.set("y".to_string(), PropertyValue::String("hello".to_string()));

        assert_eq!(obj.len(), 2);

        if let Some(PropertyValue::Number(n)) = obj.get("x") {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_js_object_trace_refs() {
        let child_ref = GcRef::new_young(1);
        let proto_ref = GcRef::new_old(0);

        let mut obj = JsObject::with_prototype(proto_ref);
        obj.set("child".to_string(), PropertyValue::Object(child_ref));

        let refs = obj.trace_refs();
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&proto_ref));
        assert!(refs.contains(&child_ref));
    }

    #[test]
    fn test_gc_object_size() {
        let mut obj = JsObject::new();
        obj.set(
            "key".to_string(),
            PropertyValue::String("value".to_string()),
        );

        let gc_obj = GcObject::new(obj);
        assert!(gc_obj.size() > 0);
    }
}
