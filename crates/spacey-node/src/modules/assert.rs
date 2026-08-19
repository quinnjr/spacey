// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `assert` module implementation

use crate::error::{NodeError, Result};
use crate::modules::util;
use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the assert module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Strict mode
    exports.insert("strict".to_string(), Value::Undefined);

    // AssertionError class
    exports.insert("AssertionError".to_string(), Value::Undefined);

    Value::NativeObject(exports)
}

/// assert(value, message?) - assert that value is truthy
pub fn assert(value: &Value, message: Option<&str>) -> Result<()> {
    if !is_truthy(value) {
        Err(NodeError::Assertion(
            message
                .unwrap_or("The expression evaluated to a falsy value")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.ok(value, message?) - same as assert()
pub fn ok(value: &Value, message: Option<&str>) -> Result<()> {
    assert(value, message)
}

/// assert.equal(actual, expected, message?) - loose equality (==)
pub fn equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if !loose_equal(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are not equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.notEqual(actual, expected, message?) - loose inequality (!=)
pub fn not_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if loose_equal(actual, expected) {
        Err(NodeError::Assertion(
            message.unwrap_or_else(|| "Values are equal").to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.strictEqual(actual, expected, message?) - strict equality (===)
pub fn strict_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if !strict_equal_values(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are not strictly equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.notStrictEqual(actual, expected, message?) - strict inequality (!==)
pub fn not_strict_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if strict_equal_values(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are strictly equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.deepEqual(actual, expected, message?) - deep loose equality
pub fn deep_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if !deep_loose_equal(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are not deeply equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.notDeepEqual(actual, expected, message?)
pub fn not_deep_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if deep_loose_equal(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are deeply equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.deepStrictEqual(actual, expected, message?) - deep strict equality
pub fn deep_strict_equal(actual: &Value, expected: &Value, message: Option<&str>) -> Result<()> {
    if !util::is_deep_strict_equal(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are not deeply strictly equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.notDeepStrictEqual(actual, expected, message?)
pub fn not_deep_strict_equal(
    actual: &Value,
    expected: &Value,
    message: Option<&str>,
) -> Result<()> {
    if util::is_deep_strict_equal(actual, expected) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "Values are deeply strictly equal")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.fail(message?)
pub fn fail(message: Option<&str>) -> Result<()> {
    Err(NodeError::Assertion(
        message.unwrap_or("Failed").to_string(),
    ))
}

/// assert.ifError(value) - throws if value is truthy
pub fn if_error(value: &Value) -> Result<()> {
    if is_truthy(value) {
        Err(NodeError::Assertion(format!(
            "Got unwanted error: {:?}",
            value
        )))
    } else {
        Ok(())
    }
}

/// assert.match(string, regexp, message?) - check if string matches regexp
pub fn matches(string: &str, pattern: &str, message: Option<&str>) -> Result<()> {
    let re = regex::Regex::new(pattern)
        .map_err(|e| NodeError::Assertion(format!("Invalid regex: {}", e)))?;

    if !re.is_match(string) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "String does not match pattern")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

/// assert.doesNotMatch(string, regexp, message?)
pub fn does_not_match(string: &str, pattern: &str, message: Option<&str>) -> Result<()> {
    let re = regex::Regex::new(pattern)
        .map_err(|e| NodeError::Assertion(format!("Invalid regex: {}", e)))?;

    if re.is_match(string) {
        Err(NodeError::Assertion(
            message
                .unwrap_or_else(|| "String matches pattern")
                .to_string(),
        ))
    } else {
        Ok(())
    }
}

// Helper functions

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0 && !n.is_nan(),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn loose_equal(a: &Value, b: &Value) -> bool {
    // Simplified loose equality
    match (a, b) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Undefined, Value::Null) | (Value::Null, Value::Undefined) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
            s.parse::<f64>().map(|sn| sn == *n).unwrap_or(false)
        }
        _ => false,
    }
}

fn strict_equal_values(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

fn deep_loose_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        // Object comparison (arrays are objects)
        (Value::Object(a), Value::Object(b)) => a == b,
        (Value::NativeObject(a), Value::NativeObject(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter()
                .all(|(k, v)| b.get(k).map(|bv| deep_loose_equal(v, bv)).unwrap_or(false))
        }
        _ => loose_equal(a, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_truthy() {
        assert!(assert(&Value::Boolean(true), None).is_ok());
        assert!(assert(&Value::Number(1.0), None).is_ok());
        assert!(assert(&Value::String("hello".to_string()), None).is_ok());

        assert!(assert(&Value::Boolean(false), None).is_err());
        assert!(assert(&Value::Number(0.0), None).is_err());
        assert!(assert(&Value::String("".to_string()), None).is_err());
        assert!(assert(&Value::Null, None).is_err());
        assert!(assert(&Value::Undefined, None).is_err());
    }

    #[test]
    fn test_strict_equal() {
        assert!(strict_equal(&Value::Number(1.0), &Value::Number(1.0), None).is_ok());
        assert!(
            strict_equal(
                &Value::String("a".to_string()),
                &Value::String("a".to_string()),
                None
            )
            .is_ok()
        );

        assert!(strict_equal(&Value::Number(1.0), &Value::String("1".to_string()), None).is_err());
    }

    #[test]
    fn test_deep_strict_equal() {
        // Create array-like objects (arrays are represented as NativeObject in spacey)
        fn make_array(values: Vec<Value>) -> Value {
            let mut obj = std::collections::HashMap::new();
            for (i, v) in values.into_iter().enumerate() {
                obj.insert(i.to_string(), v);
            }
            obj.insert(
                "length".to_string(),
                Value::Number(obj.len() as f64 - 1.0 + 1.0),
            );
            Value::NativeObject(obj)
        }

        let arr1 = make_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let arr2 = make_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let arr3 = make_array(vec![Value::Number(1.0), Value::Number(3.0)]);

        assert!(deep_strict_equal(&arr1, &arr2, None).is_ok());
        assert!(deep_strict_equal(&arr1, &arr3, None).is_err());
    }

    #[test]
    fn test_fail() {
        assert!(fail(None).is_err());
        assert!(fail(Some("Custom message")).is_err());
    }
}
