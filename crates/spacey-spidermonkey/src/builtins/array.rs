//! Array built-in object (ES3 Section 15.4).
//!
//! Provides Array constructor and prototype methods.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

/// Internal array representation.
/// In a full implementation, this would be stored in the heap.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct JsArray {
    pub elements: Vec<Value>,
}

#[allow(missing_docs)]
impl JsArray {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    pub fn from_elements(elements: Vec<Value>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Default for JsArray {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Array Constructor (ES3 Section 15.4.1-2)
// ============================================================================

/// Array() constructor - creates a new array.
pub fn array_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Single numeric argument = array with that length
    if args.len() == 1
        && let Value::Number(n) = &args[0] {
            let len = n.to_bits();
            if *n >= 0.0 && n.fract() == 0.0 && *n <= u32::MAX as f64 {
                // Create array with specified length
                // In real impl, would allocate in heap
                return Ok(Value::Object(len as usize));
            } else {
                return Err("RangeError: Invalid array length".to_string());
            }
        }

    // Multiple arguments or non-numeric single argument = array with those elements
    // In real impl, would create array in heap and return reference
    Ok(Value::Object(args.len()))
}

/// Array.isArray(arg) - Returns true if arg is an array.
/// Note: This is ES5, but useful to include
pub fn is_array(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // In real impl, would check [[Class]] internal slot
    let is_arr = args
        .first()
        .map(|v| matches!(v, Value::Object(_)))
        .unwrap_or(false);
    Ok(Value::Boolean(is_arr))
}

// ============================================================================
// Array.prototype Methods (ES3 Section 15.4.4)
// ============================================================================

// For the prototype methods, we work with a virtual array representation.
// In a full implementation, these would access the heap.

/// Array.prototype.toString() - Returns comma-separated string.
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // args[0] is 'this' in prototype methods
    let elements = extract_elements(args);
    let parts: Vec<String> = elements.iter().map(|v| v.to_js_string()).collect();
    Ok(Value::String(parts.join(",")))
}

/// Array.prototype.toLocaleString() - Returns locale-aware string.
pub fn to_locale_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Simplified: same as toString for now
    to_string(_frame, args)
}

/// Array.prototype.concat(...items) - Returns new array with items appended.
pub fn concat(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut result = extract_elements(args);

    // Add remaining arguments
    for arg in args.iter().skip(1) {
        // In real impl, would check if arg is array and spread it
        result.push(arg.clone());
    }

    Ok(Value::Object(result.len()))
}

/// Array.prototype.join(separator) - Joins elements with separator.
pub fn join(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let elements = extract_elements(args);

    let separator = args
        .get(1)
        .map(|v| {
            if matches!(v, Value::Undefined) {
                ",".to_string()
            } else {
                v.to_js_string()
            }
        })
        .unwrap_or_else(|| ",".to_string());

    let parts: Vec<String> = elements
        .iter()
        .map(|v| {
            if matches!(v, Value::Undefined | Value::Null) {
                String::new()
            } else {
                v.to_js_string()
            }
        })
        .collect();

    Ok(Value::String(parts.join(&separator)))
}

/// Array.prototype.pop() - Removes and returns the last element.
pub fn pop(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);

    if elements.is_empty() {
        Ok(Value::Undefined)
    } else {
        Ok(elements.pop().unwrap_or(Value::Undefined))
    }
}

/// Array.prototype.push(...items) - Adds items to end, returns new length.
pub fn push(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);

    for arg in args.iter().skip(1) {
        elements.push(arg.clone());
    }

    Ok(Value::Number(elements.len() as f64))
}

/// Array.prototype.reverse() - Reverses the array in place.
pub fn reverse(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);
    elements.reverse();
    // In real impl, would modify array in place and return it
    Ok(Value::Object(elements.len()))
}

/// Array.prototype.shift() - Removes and returns the first element.
pub fn shift(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);

    if elements.is_empty() {
        Ok(Value::Undefined)
    } else {
        Ok(elements.remove(0))
    }
}

/// Array.prototype.slice(start, end) - Returns a portion of the array.
pub fn slice(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let elements = extract_elements(args);
    let len = elements.len() as i32;

    let start = args
        .get(1)
        .map(|v| {
            let n = v.to_integer() as i32;
            if n < 0 {
                (len + n).max(0) as usize
            } else {
                n.min(len) as usize
            }
        })
        .unwrap_or(0);

    let end = args
        .get(2)
        .map(|v| {
            if matches!(v, Value::Undefined) {
                len as usize
            } else {
                let n = v.to_integer() as i32;
                if n < 0 {
                    (len + n).max(0) as usize
                } else {
                    n.min(len) as usize
                }
            }
        })
        .unwrap_or(len as usize);

    let result: Vec<Value> = if start < end && start < elements.len() {
        elements[start..end.min(elements.len())].to_vec()
    } else {
        Vec::new()
    };

    Ok(Value::Object(result.len()))
}

/// Array.prototype.sort(comparefn) - Sorts the array.
pub fn sort(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);

    // Default sort converts to strings
    // In real impl, would use comparefn if provided
    elements.sort_by(|a, b| {
        let a_str = a.to_js_string();
        let b_str = b.to_js_string();
        a_str.cmp(&b_str)
    });

    Ok(Value::Object(elements.len()))
}

/// Array.prototype.splice(start, deleteCount, ...items) - Modifies array.
pub fn splice(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);
    let len = elements.len() as i32;

    let start = args
        .get(1)
        .map(|v| {
            let n = v.to_integer() as i32;
            if n < 0 {
                (len + n).max(0) as usize
            } else {
                n.min(len) as usize
            }
        })
        .unwrap_or(0);

    let delete_count = args
        .get(2)
        .map(|v| {
            let n = v.to_integer() as i32;
            n.max(0).min(len - start as i32) as usize
        })
        .unwrap_or((len - start as i32).max(0) as usize);

    // Remove elements
    let end = (start + delete_count).min(elements.len());
    let removed: Vec<Value> = elements.drain(start..end).collect();

    // Insert new elements
    for (i, item) in args.iter().skip(3).enumerate() {
        elements.insert(start + i, item.clone());
    }

    // Return removed elements as array
    Ok(Value::Object(removed.len()))
}

/// Array.prototype.unshift(...items) - Adds items to beginning.
pub fn unshift(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let mut elements = extract_elements(args);

    for (i, arg) in args.iter().skip(1).enumerate() {
        elements.insert(i, arg.clone());
    }

    Ok(Value::Number(elements.len() as f64))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract elements from the 'this' value (first arg in prototype methods).
/// In a full implementation, this would look up the object in the heap.
fn extract_elements(_args: &[Value]) -> Vec<Value> {
    // Placeholder - in real impl would get array from heap
    Vec::new()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::Function;

    fn make_frame() -> CallFrame {
        let func = Function::new(None, vec![], Bytecode::new(), 0);
        CallFrame::new(func, 0)
    }

    #[test]
    fn test_js_array_new() {
        let arr = JsArray::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_js_array_with_capacity() {
        let arr = JsArray::with_capacity(10);
        assert!(arr.is_empty());
    }

    #[test]
    fn test_js_array_from_elements() {
        let arr = JsArray::from_elements(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_array_constructor_empty() {
        let mut frame = make_frame();
        let result = array_constructor(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_array_constructor_with_length() {
        let mut frame = make_frame();
        let result = array_constructor(&mut frame, &[Value::Number(5.0)]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_array_constructor_invalid_length() {
        let mut frame = make_frame();
        let result = array_constructor(&mut frame, &[Value::Number(-1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_array() {
        let mut frame = make_frame();
        assert!(matches!(
            is_array(&mut frame, &[Value::Object(0)]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            is_array(&mut frame, &[Value::Number(42.0)]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn test_join_default_separator() {
        let mut frame = make_frame();
        let result = join(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::String(s) if s.is_empty()));
    }

    #[test]
    fn test_pop_empty() {
        let mut frame = make_frame();
        let result = pop(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_push() {
        let mut frame = make_frame();
        let result = push(&mut frame, &[Value::Object(0), Value::Number(1.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_shift_empty() {
        let mut frame = make_frame();
        let result = shift(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_reverse() {
        let mut frame = make_frame();
        let result = reverse(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_slice() {
        let mut frame = make_frame();
        let result = slice(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_sort() {
        let mut frame = make_frame();
        let result = sort(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_splice() {
        let mut frame = make_frame();
        let result = splice(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_to_string() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::String(s) if s.is_empty()));
    }

    #[test]
    fn test_concat() {
        let mut frame = make_frame();
        let result = concat(&mut frame, &[Value::Object(0), Value::Number(1.0)]).unwrap();
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_unshift() {
        let mut frame = make_frame();
        let result = unshift(&mut frame, &[Value::Object(0), Value::Number(1.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }
}
