//! Runtime object representation for the VM.

use std::collections::HashMap;
use crate::runtime::value::Value;

/// Runtime object representation
#[derive(Clone, Debug)]
pub struct RuntimeObject {
    /// Properties stored as key-value pairs
    properties: HashMap<String, Value>,
    /// For arrays: stores elements by index
    array_elements: Vec<Value>,
    /// Whether this is an array
    is_array: bool,
}

impl RuntimeObject {
    /// Create a new empty object
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            array_elements: Vec::new(),
            is_array: false,
        }
    }

    /// Create a new array with elements
    pub fn new_array(elements: Vec<Value>) -> Self {
        let len = elements.len();
        let mut obj = Self {
            properties: HashMap::new(),
            array_elements: elements,
            is_array: true,
        };
        obj.properties.insert("length".to_string(), Value::Number(len as f64));
        obj
    }

    /// Get a property by name
    pub fn get(&self, name: &str) -> Value {
        if self.is_array {
            // Check for numeric index
            if let Ok(idx) = name.parse::<usize>() {
                return self.array_elements.get(idx).cloned().unwrap_or(Value::Undefined);
            }
        }
        self.properties.get(name).cloned().unwrap_or(Value::Undefined)
    }

    /// Check if this is an array
    pub fn is_array(&self) -> bool {
        self.is_array
    }

    /// Get array elements (for array methods)
    pub fn elements(&self) -> &[Value] {
        &self.array_elements
    }

    /// Push an element to the array, returns new length
    pub fn array_push(&mut self, value: Value) -> f64 {
        if self.is_array {
            self.array_elements.push(value);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            len
        } else {
            0.0
        }
    }

    /// Pop an element from the array
    pub fn array_pop(&mut self) -> Value {
        if self.is_array {
            let result = self.array_elements.pop().unwrap_or(Value::Undefined);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            result
        } else {
            Value::Undefined
        }
    }

    /// Shift (remove first element)
    pub fn array_shift(&mut self) -> Value {
        if self.is_array && !self.array_elements.is_empty() {
            let result = self.array_elements.remove(0);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            result
        } else {
            Value::Undefined
        }
    }

    /// Unshift (add to beginning), returns new length
    pub fn array_unshift(&mut self, value: Value) -> f64 {
        if self.is_array {
            self.array_elements.insert(0, value);
            let len = self.array_elements.len() as f64;
            self.properties.insert("length".to_string(), Value::Number(len));
            len
        } else {
            0.0
        }
    }

    /// Join elements with separator
    pub fn array_join(&self, separator: &str) -> String {
        if self.is_array {
            self.array_elements
                .iter()
                .map(|v| v.to_js_string())
                .collect::<Vec<_>>()
                .join(separator)
        } else {
            String::new()
        }
    }

    /// Reverse the array in place
    pub fn array_reverse(&mut self) {
        if self.is_array {
            self.array_elements.reverse();
        }
    }

    /// Slice the array
    pub fn array_slice(&self, start: i32, end: i32) -> Vec<Value> {
        if self.is_array {
            let len = self.array_elements.len() as i32;
            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
            let end = if end < 0 { (len + end).max(0) } else { end.min(len) } as usize;
            if start >= end {
                vec![]
            } else {
                self.array_elements[start..end].to_vec()
            }
        } else {
            vec![]
        }
    }

    /// Index of an element
    pub fn array_index_of(&self, value: &Value) -> i32 {
        if self.is_array {
            for (i, elem) in self.array_elements.iter().enumerate() {
                if elem == value {
                    return i as i32;
                }
            }
        }
        -1
    }

    /// Concat arrays
    pub fn array_concat(&self, other: &[Value]) -> Vec<Value> {
        if self.is_array {
            let mut result = self.array_elements.clone();
            result.extend(other.iter().cloned());
            result
        } else {
            other.to_vec()
        }
    }

    /// Set a property
    pub fn set(&mut self, name: &str, value: Value) {
        if self.is_array
            && let Ok(idx) = name.parse::<usize>() {
                // Extend array if necessary
                while self.array_elements.len() <= idx {
                    self.array_elements.push(Value::Undefined);
                }
                self.array_elements[idx] = value;
                // Update length
                self.properties.insert("length".to_string(), Value::Number(self.array_elements.len() as f64));
                return;
            }
        self.properties.insert(name.to_string(), value);
    }

    /// Get all enumerable property keys
    pub fn keys(&self) -> Vec<String> {
        if self.is_array {
            // For arrays, return indices as strings plus property keys
            let mut keys: Vec<String> = (0..self.array_elements.len())
                .map(|i| i.to_string())
                .collect();
            // Add non-index properties (except length)
            for key in self.properties.keys() {
                if key != "length" && key.parse::<usize>().is_err() {
                    keys.push(key.clone());
                }
            }
            keys
        } else {
            // For regular objects, return all property keys
            self.properties.keys().cloned().collect()
        }
    }

    /// Delete a property
    pub fn delete(&mut self, name: &str) -> bool {
        if self.is_array
            && let Ok(idx) = name.parse::<usize>()
                && idx < self.array_elements.len() {
                    self.array_elements[idx] = Value::Undefined;
                    return true;
                }
        self.properties.remove(name).is_some()
    }
}

impl Default for RuntimeObject {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object() {
        let obj = RuntimeObject::new();
        assert!(!obj.is_array());
        assert!(obj.keys().is_empty());
    }

    #[test]
    fn test_new_array() {
        let arr = RuntimeObject::new_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert!(arr.is_array());
        assert_eq!(arr.elements().len(), 2);
    }

    #[test]
    fn test_get_set() {
        let mut obj = RuntimeObject::new();
        obj.set("foo", Value::Number(42.0));
        assert!(matches!(obj.get("foo"), Value::Number(n) if n == 42.0));
        assert!(matches!(obj.get("bar"), Value::Undefined));
    }

    #[test]
    fn test_array_push_pop() {
        let mut arr = RuntimeObject::new_array(vec![]);
        assert_eq!(arr.array_push(Value::Number(1.0)), 1.0);
        assert_eq!(arr.array_push(Value::Number(2.0)), 2.0);
        assert!(matches!(arr.array_pop(), Value::Number(n) if n == 2.0));
        assert!(matches!(arr.array_pop(), Value::Number(n) if n == 1.0));
        assert!(matches!(arr.array_pop(), Value::Undefined));
    }

    #[test]
    fn test_array_shift_unshift() {
        let mut arr = RuntimeObject::new_array(vec![Value::Number(2.0)]);
        assert_eq!(arr.array_unshift(Value::Number(1.0)), 2.0);
        assert!(matches!(arr.array_shift(), Value::Number(n) if n == 1.0));
        assert!(matches!(arr.array_shift(), Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_array_join() {
        let arr = RuntimeObject::new_array(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]);
        assert_eq!(arr.array_join(","), "a,b,c");
        assert_eq!(arr.array_join("-"), "a-b-c");
    }

    #[test]
    fn test_array_slice() {
        let arr = RuntimeObject::new_array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let sliced = arr.array_slice(1, 3);
        assert_eq!(sliced.len(), 2);
    }

    #[test]
    fn test_array_index_of() {
        let arr = RuntimeObject::new_array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(arr.array_index_of(&Value::Number(2.0)), 1);
        assert_eq!(arr.array_index_of(&Value::Number(5.0)), -1);
    }

    #[test]
    fn test_array_reverse() {
        let mut arr = RuntimeObject::new_array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        arr.array_reverse();
        assert!(matches!(arr.elements()[0], Value::Number(n) if n == 3.0));
        assert!(matches!(arr.elements()[2], Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_delete() {
        let mut obj = RuntimeObject::new();
        obj.set("foo", Value::Number(42.0));
        assert!(obj.delete("foo"));
        assert!(matches!(obj.get("foo"), Value::Undefined));
        assert!(!obj.delete("bar"));
    }
}



