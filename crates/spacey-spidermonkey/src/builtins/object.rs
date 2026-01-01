//! Object built-in constructor and prototype methods.
//!
//! Provides the Object constructor and Object.prototype methods.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

/// Object() constructor - converts value to object or creates empty object.
pub fn object_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args[0].is_nullish() {
        // Create new empty object
        Ok(Value::Object(0)) // Placeholder - would be actual object
    } else {
        // Return the value (would convert primitives to wrapper objects)
        Ok(args[0].clone())
    }
}

/// Object.keys() - returns array of own enumerable property names.
pub fn object_keys(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper object property enumeration
    Ok(Value::Object(0)) // Returns empty array placeholder
}

/// Object.values() - returns array of own enumerable property values.
pub fn object_values(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper object property enumeration
    Ok(Value::Object(0)) // Returns empty array placeholder
}

/// Object.entries() - returns array of [key, value] pairs.
pub fn object_entries(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper object property enumeration
    Ok(Value::Object(0)) // Returns empty array placeholder
}

/// Object.assign() - copies properties from source to target.
pub fn object_assign(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Object.assign requires at least one argument".to_string());
    }
    // TODO: Implement proper property copying
    Ok(args[0].clone())
}

/// Object.create() - creates object with specified prototype.
pub fn object_create(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement prototype chain
    Ok(Value::Object(0))
}

/// Object.freeze() - freezes an object.
pub fn object_freeze(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Object.freeze requires an argument".to_string());
    }
    // TODO: Implement object freezing
    Ok(args[0].clone())
}

/// Object.seal() - seals an object.
pub fn object_seal(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Object.seal requires an argument".to_string());
    }
    // TODO: Implement object sealing
    Ok(args[0].clone())
}

/// Object.isFrozen() - checks if object is frozen.
pub fn object_is_frozen(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper frozen check
    Ok(Value::Boolean(false))
}

/// Object.isSealed() - checks if object is sealed.
pub fn object_is_sealed(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper sealed check
    Ok(Value::Boolean(false))
}

/// Object.prototype.hasOwnProperty() - checks if property exists on object.
pub fn object_has_own_property(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // TODO: Implement proper property check
    Ok(Value::Boolean(false))
}

/// Object.prototype.toString() - returns string representation.
pub fn object_to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // ES3 Section 15.2.4.2
    if args.is_empty() {
        return Ok(Value::String("[object Undefined]".to_string()));
    }

    let type_tag = match &args[0] {
        Value::Undefined => "Undefined",
        Value::Null => "Null",
        Value::Boolean(_) => "Boolean",
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Object(_) => "Object",
        Value::Function(_) => "Function",
        Value::NativeObject(_) => "Object",
        Value::Symbol(_) => "Symbol",
        Value::BigInt(_) => "BigInt",
    };

    Ok(Value::String(format!("[object {}]", type_tag)))
}

/// Object.prototype.toLocaleString() - returns locale-specific string.
/// ES3 Section 15.2.4.3
pub fn object_to_locale_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Default implementation just calls toString()
    object_to_string(_frame, args)
}

/// Object.prototype.valueOf() - returns primitive value.
pub fn object_value_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        Ok(Value::Undefined)
    } else {
        Ok(args[0].clone())
    }
}

/// Object.prototype.isPrototypeOf(V) - checks if object is in prototype chain.
/// ES3 Section 15.2.4.6
pub fn object_is_prototype_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // For ES3 compliance, we need the actual prototype chain implementation
    // For now, return false for non-objects, and check basic cases
    let this_obj = args.first().unwrap_or(&Value::Undefined);
    let target = args.get(1).unwrap_or(&Value::Undefined);

    match (this_obj, target) {
        (_, Value::Undefined | Value::Null | Value::Boolean(_) | Value::Number(_) | Value::String(_)) => {
            // Primitives have no prototype chain to walk
            Ok(Value::Boolean(false))
        }
        (Value::Object(_), Value::Object(_)) => {
            // In a full implementation, we'd walk the prototype chain
            // For now, return false (objects don't inherit from each other by default)
            Ok(Value::Boolean(false))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

/// Object.prototype.propertyIsEnumerable(V) - checks if property is enumerable.
/// ES3 Section 15.2.4.7
pub fn object_property_is_enumerable(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let this_obj = args.first().unwrap_or(&Value::Undefined);
    let prop_name = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();

    match this_obj {
        Value::Object(_) | Value::NativeObject(_) => {
            // In a full implementation, check the property descriptor
            // All own properties in our implementation are enumerable by default
            if let Value::NativeObject(props) = this_obj {
                Ok(Value::Boolean(props.contains_key(&prop_name)))
            } else {
                // For regular objects, would need heap access
                Ok(Value::Boolean(false))
            }
        }
        Value::String(s) => {
            // String indices are enumerable, length and other properties are not
            if let Ok(idx) = prop_name.parse::<usize>() {
                Ok(Value::Boolean(idx < s.len()))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Ok(Value::Boolean(false)),
    }
}

/// typeof operator implementation.
pub fn typeof_value(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::String("undefined".to_string()));
    }
    Ok(Value::String(args[0].type_of().to_string()))
}

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
    fn test_object_constructor_no_args() {
        let mut frame = make_frame();
        let result = object_constructor(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Object(_)));
    }

    #[test]
    fn test_object_constructor_null() {
        let mut frame = make_frame();
        let result = object_constructor(&mut frame, &[Value::Null]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Object(_)));
    }

    #[test]
    fn test_object_constructor_undefined() {
        let mut frame = make_frame();
        let result = object_constructor(&mut frame, &[Value::Undefined]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Object(_)));
    }

    #[test]
    fn test_object_constructor_with_value() {
        let mut frame = make_frame();
        let result = object_constructor(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_object_keys() {
        let mut frame = make_frame();
        let result = object_keys(&mut frame, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_values() {
        let mut frame = make_frame();
        let result = object_values(&mut frame, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_entries() {
        let mut frame = make_frame();
        let result = object_entries(&mut frame, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_assign_no_args() {
        let mut frame = make_frame();
        let result = object_assign(&mut frame, &[]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("requires at least one argument")
        );
    }

    #[test]
    fn test_object_assign_with_target() {
        let mut frame = make_frame();
        let result = object_assign(&mut frame, &[Value::Object(1)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_create() {
        let mut frame = make_frame();
        let result = object_create(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Object(_)));
    }

    #[test]
    fn test_object_freeze_no_args() {
        let mut frame = make_frame();
        let result = object_freeze(&mut frame, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires an argument"));
    }

    #[test]
    fn test_object_freeze_with_object() {
        let mut frame = make_frame();
        let result = object_freeze(&mut frame, &[Value::Object(1)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_seal_no_args() {
        let mut frame = make_frame();
        let result = object_seal(&mut frame, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires an argument"));
    }

    #[test]
    fn test_object_seal_with_object() {
        let mut frame = make_frame();
        let result = object_seal(&mut frame, &[Value::Object(1)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_is_frozen() {
        let mut frame = make_frame();
        let result = object_is_frozen(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(false)));
    }

    #[test]
    fn test_object_is_sealed() {
        let mut frame = make_frame();
        let result = object_is_sealed(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(false)));
    }

    #[test]
    fn test_object_has_own_property() {
        let mut frame = make_frame();
        let result = object_has_own_property(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(false)));
    }

    #[test]
    fn test_object_to_string() {
        let mut frame = make_frame();
        // With no args, returns [object Undefined]
        let result = object_to_string(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "[object Undefined]"));
    }

    #[test]
    fn test_object_to_string_with_object() {
        let mut frame = make_frame();
        let result = object_to_string(&mut frame, &[Value::Object(0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "[object Object]"));
    }

    #[test]
    fn test_object_to_string_with_number() {
        let mut frame = make_frame();
        let result = object_to_string(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "[object Number]"));
    }

    #[test]
    fn test_object_value_of_no_args() {
        let mut frame = make_frame();
        let result = object_value_of(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_object_value_of_with_value() {
        let mut frame = make_frame();
        let result = object_value_of(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_typeof_value_no_args() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "undefined"));
    }

    #[test]
    fn test_typeof_value_number() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "number"));
    }

    #[test]
    fn test_typeof_value_string() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[Value::String("test".to_string())]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "string"));
    }

    #[test]
    fn test_typeof_value_boolean() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[Value::Boolean(true)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "boolean"));
    }

    #[test]
    fn test_typeof_value_object() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[Value::Object(0)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::String(s) if s == "object"));
    }

    #[test]
    fn test_typeof_value_null() {
        let mut frame = make_frame();
        let result = typeof_value(&mut frame, &[Value::Null]);
        assert!(result.is_ok());
        // null returns "object" (historical quirk)
        assert!(matches!(result.unwrap(), Value::String(s) if s == "object"));
    }
}
