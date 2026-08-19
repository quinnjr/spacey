//! Abstract equality comparison (ES3 Section 11.9.3)

use crate::runtime::value::Value;
use std::sync::Arc;

/// Abstract equality comparison (ES3 Section 11.9.3)
///
/// The Abstract Equality Comparison Algorithm with type coercion.
pub fn abstract_equals(a: &Value, b: &Value) -> bool {
    // 1. If Type(x) is the same as Type(y), return strict equality
    match (a, b) {
        // Same type comparisons
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // NaN != NaN in JavaScript
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a == b
            }
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Object(a), Value::Object(b)) => a == b,
        (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b),

        // 2. null == undefined is true
        (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,

        // 3. If one is number and other is string, convert string to number
        (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
            let s_num = s.parse::<f64>().unwrap_or(f64::NAN);
            if s_num.is_nan() || n.is_nan() {
                false
            } else {
                n == &s_num
            }
        }

        // 4. If one is boolean, convert it to number and compare
        (Value::Boolean(b_val), other) => {
            let num = if *b_val { 1.0 } else { 0.0 };
            abstract_equals(&Value::Number(num), other)
        }
        (other, Value::Boolean(b_val)) => {
            let num = if *b_val { 1.0 } else { 0.0 };
            abstract_equals(other, &Value::Number(num))
        }

        // 5. If one is number/string and other is object, convert object to primitive
        // For now, simplified handling
        (Value::Number(_) | Value::String(_), Value::Object(_)) => {
            // ToPrimitive(object) - simplified, compare as false
            false
        }
        (Value::Object(_), Value::Number(_) | Value::String(_)) => {
            // ToPrimitive(object) - simplified, compare as false
            false
        }

        // All other cases are not equal
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstract_equals_same_type() {
        assert!(abstract_equals(&Value::Undefined, &Value::Undefined));
        assert!(abstract_equals(&Value::Null, &Value::Null));
        assert!(abstract_equals(
            &Value::Boolean(true),
            &Value::Boolean(true)
        ));
        assert!(!abstract_equals(
            &Value::Boolean(true),
            &Value::Boolean(false)
        ));
        assert!(abstract_equals(&Value::Number(42.0), &Value::Number(42.0)));
        assert!(abstract_equals(
            &Value::String("foo".into()),
            &Value::String("foo".into())
        ));
    }

    #[test]
    fn test_abstract_equals_null_undefined() {
        assert!(abstract_equals(&Value::Null, &Value::Undefined));
        assert!(abstract_equals(&Value::Undefined, &Value::Null));
    }

    #[test]
    fn test_abstract_equals_number_string() {
        assert!(abstract_equals(
            &Value::Number(42.0),
            &Value::String("42".into())
        ));
        assert!(abstract_equals(
            &Value::String("42".into()),
            &Value::Number(42.0)
        ));
        assert!(!abstract_equals(
            &Value::Number(42.0),
            &Value::String("43".into())
        ));
    }

    #[test]
    fn test_abstract_equals_boolean_coercion() {
        assert!(abstract_equals(&Value::Boolean(true), &Value::Number(1.0)));
        assert!(abstract_equals(&Value::Boolean(false), &Value::Number(0.0)));
        assert!(abstract_equals(&Value::Number(1.0), &Value::Boolean(true)));
    }

    #[test]
    fn test_abstract_equals_nan() {
        assert!(!abstract_equals(
            &Value::Number(f64::NAN),
            &Value::Number(f64::NAN)
        ));
    }
}
