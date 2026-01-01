//! JavaScript value representation.

use super::function::Callable;
use std::fmt;
use std::sync::Arc;

/// A JavaScript value.
///
/// Values are designed to be thread-safe and can be safely shared
/// between async tasks.
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum Value {
    /// undefined
    #[default]
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number (IEEE 754 double)
    Number(f64),
    /// String
    String(String),
    /// Symbol
    Symbol(u64),
    /// BigInt (stored as string for now)
    BigInt(String),
    /// Object reference (placeholder - would be GC handle)
    Object(usize),
    /// Function reference (Arc for thread safety)
    Function(Arc<Callable>),
    /// Native object with properties (for built-in objects like console, Math)
    NativeObject(std::collections::HashMap<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => {
                // Handle NaN comparisons
                if a.is_nan() && b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b),
            (Value::NativeObject(_), Value::NativeObject(_)) => false, // Native objects are not equal by value
            _ => false,
        }
    }
}

impl Value {
    /// Returns true if this value is undefined.
    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    /// Returns true if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if this value is nullish (null or undefined).
    pub fn is_nullish(&self) -> bool {
        matches!(self, Value::Undefined | Value::Null)
    }

    /// Returns true if this value is a function.
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function(_))
    }

    /// Converts the value to a boolean (ToBoolean).
    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => !n.is_nan() && *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Symbol(_)
            | Value::BigInt(_)
            | Value::Object(_)
            | Value::Function(_)
            | Value::NativeObject(_) => true,
        }
    }

    /// Returns the type of this value as a string.
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object", // Historical quirk
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::BigInt(_) => "bigint",
            Value::Object(_) | Value::NativeObject(_) => "object",
            Value::Function(_) => "function",
        }
    }

    // =========================================================================
    // ES3 Type Conversion Operations (Section 9)
    // =========================================================================

    /// ToNumber (ES3 Section 9.3)
    ///
    /// Converts the value to a number.
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Number(n) => *n,
            Value::String(s) => Self::string_to_number(s),
            Value::BigInt(_) => f64::NAN, // Would throw in real ES
            Value::Symbol(_) => f64::NAN, // Would throw in real ES
            Value::Object(_) | Value::NativeObject(_) => f64::NAN, // Would call ToPrimitive first
            Value::Function(_) => f64::NAN,
        }
    }

    /// ToString (ES3 Section 9.8)
    ///
    /// Converts the value to a string.
    pub fn to_js_string(&self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Number(n) => Self::number_to_string(*n),
            Value::String(s) => s.clone(),
            Value::BigInt(s) => s.clone(),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::Object(_) | Value::NativeObject(_) => "[object Object]".to_string(),
            Value::Function(callable) => match callable.as_ref() {
                Callable::Function(func) => {
                    if let Some(name) = &func.name {
                        format!("function {}() {{ [native code] }}", name)
                    } else {
                        "function () { [native code] }".to_string()
                    }
                }
                Callable::Native { name, .. } => {
                    format!("function {}() {{ [native code] }}", name)
                }
            },
        }
    }

    /// ToInteger (ES3 Section 9.4)
    ///
    /// Converts the value to an integer.
    pub fn to_integer(&self) -> f64 {
        let num = self.to_number();
        if num.is_nan() {
            return 0.0;
        }
        if num == 0.0 || num.is_infinite() {
            return num;
        }
        num.signum() * num.abs().floor()
    }

    /// ToInt32 (ES3 Section 9.5)
    ///
    /// Converts the value to a signed 32-bit integer.
    pub fn to_int32(&self) -> i32 {
        let num = self.to_number();
        if num.is_nan() || num == 0.0 || num.is_infinite() {
            return 0;
        }
        let int = (num.signum() * num.abs().floor()) as i64;
        let int32bit = int % (1i64 << 32);
        if int32bit >= (1i64 << 31) {
            (int32bit - (1i64 << 32)) as i32
        } else {
            int32bit as i32
        }
    }

    /// ToUint32 (ES3 Section 9.6)
    ///
    /// Converts the value to an unsigned 32-bit integer.
    pub fn to_uint32(&self) -> u32 {
        let num = self.to_number();
        if num.is_nan() || num == 0.0 || num.is_infinite() {
            return 0;
        }
        let int = (num.signum() * num.abs().floor()) as i64;
        // Rust's modulo preserves sign, but we need modulo 2^32 with positive result
        let modulo = 1i64 << 32;
        let result = int % modulo;
        if result < 0 {
            (result + modulo) as u32
        } else {
            result as u32
        }
    }

    /// ToUint16 (ES3 Section 9.7)
    ///
    /// Converts the value to an unsigned 16-bit integer.
    pub fn to_uint16(&self) -> u16 {
        let num = self.to_number();
        if num.is_nan() || num == 0.0 || num.is_infinite() {
            return 0;
        }
        let int = (num.signum() * num.abs().floor()) as i64;
        (int % (1i64 << 16)) as u16
    }

    /// Helper: Convert string to number (ES3 Section 9.3.1)
    fn string_to_number(s: &str) -> f64 {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return 0.0;
        }

        // Handle hex
        if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            if let Ok(n) = i64::from_str_radix(&trimmed[2..], 16) {
                return n as f64;
            }
            return f64::NAN;
        }

        // Handle infinity
        if trimmed == "Infinity" || trimmed == "+Infinity" {
            return f64::INFINITY;
        }
        if trimmed == "-Infinity" {
            return f64::NEG_INFINITY;
        }

        // Try parsing as float
        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    }

    /// Helper: Convert number to string (ES3 Section 9.8.1)
    pub fn number_to_string(n: f64) -> String {
        if n.is_nan() {
            return "NaN".to_string();
        }
        if n == 0.0 {
            return "0".to_string();
        }
        if n.is_infinite() {
            return if n > 0.0 {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            };
        }

        // Check if it's an integer
        if n.fract() == 0.0 && n.abs() < 1e15 {
            return format!("{}", n as i64);
        }

        // General case
        format!("{}", n)
    }

    /// Returns true if this value is an object (including functions).
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_) | Value::Function(_))
    }

    /// Returns true if this value is a primitive (not an object).
    pub fn is_primitive(&self) -> bool {
        !self.is_object()
    }

    /// Returns true if this value is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if this value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns true if this value is a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Returns true if the value is NaN.
    pub fn is_nan(&self) -> bool {
        matches!(self, Value::Number(n) if n.is_nan())
    }

    /// Returns true if the value is finite (not NaN or Infinity).
    pub fn is_finite(&self) -> bool {
        matches!(self, Value::Number(n) if n.is_finite())
    }
}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Symbol(id) => write!(f, "Symbol({})", id),
            Value::BigInt(n) => write!(f, "{}n", n),
            Value::Object(_) => write!(f, "[object Object]"),
            Value::Function(callable) => match callable.as_ref() {
                Callable::Function(func) => {
                    if let Some(name) = &func.name {
                        write!(f, "[Function: {}]", name)
                    } else {
                        write!(f, "[Function (anonymous)]")
                    }
                }
                Callable::Native { name, .. } => {
                    write!(f, "[Function: {} (native)]", name)
                }
            },
            Value::NativeObject(_) => write!(f, "[object Object]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::{CallFrame, Function};

    fn make_test_function(name: Option<&str>) -> Function {
        Function::new(name.map(|s| s.to_string()), vec![], Bytecode::new(), 0)
    }

    fn native_test_fn(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
        Ok(Value::Undefined)
    }

    #[test]
    fn test_value_undefined() {
        let v = Value::Undefined;
        assert!(v.is_undefined());
        assert!(!v.is_null());
        assert!(v.is_nullish());
        assert!(!v.is_function());
        assert_eq!(v.type_of(), "undefined");
        assert!(!v.to_boolean());
        assert_eq!(v.to_string(), "undefined");
    }

    #[test]
    fn test_value_null() {
        let v = Value::Null;
        assert!(!v.is_undefined());
        assert!(v.is_null());
        assert!(v.is_nullish());
        assert!(!v.is_function());
        assert_eq!(v.type_of(), "object"); // Historical quirk
        assert!(!v.to_boolean());
        assert_eq!(v.to_string(), "null");
    }

    #[test]
    fn test_value_boolean() {
        let t = Value::Boolean(true);
        let f = Value::Boolean(false);

        assert!(!t.is_undefined());
        assert!(!t.is_null());
        assert!(!t.is_nullish());
        assert_eq!(t.type_of(), "boolean");
        assert!(t.to_boolean());
        assert_eq!(t.to_string(), "true");

        assert!(!f.to_boolean());
        assert_eq!(f.to_string(), "false");
    }

    #[test]
    fn test_value_number() {
        let zero = Value::Number(0.0);
        let pos = Value::Number(42.0);
        let neg = Value::Number(-1.5);
        let nan = Value::Number(f64::NAN);
        let inf = Value::Number(f64::INFINITY);

        assert_eq!(zero.type_of(), "number");
        assert!(!zero.to_boolean()); // 0 is falsy
        assert!(pos.to_boolean());
        assert!(neg.to_boolean());
        assert!(!nan.to_boolean()); // NaN is falsy
        assert!(inf.to_boolean());

        assert_eq!(pos.to_string(), "42");
        assert_eq!(neg.to_string(), "-1.5");
    }

    #[test]
    fn test_value_string() {
        let empty = Value::String(String::new());
        let hello = Value::String("hello".to_string());

        assert_eq!(empty.type_of(), "string");
        assert!(!empty.to_boolean()); // Empty string is falsy
        assert!(hello.to_boolean());
        assert_eq!(hello.to_string(), "hello");
    }

    #[test]
    fn test_value_symbol() {
        let sym = Value::Symbol(123);
        assert_eq!(sym.type_of(), "symbol");
        assert!(sym.to_boolean()); // Symbols are truthy
        assert_eq!(sym.to_string(), "Symbol(123)");
    }

    #[test]
    fn test_value_bigint() {
        let big = Value::BigInt("12345678901234567890".to_string());
        assert_eq!(big.type_of(), "bigint");
        assert!(big.to_boolean()); // BigInts are truthy
        assert_eq!(big.to_string(), "12345678901234567890n");
    }

    #[test]
    fn test_value_object() {
        let obj = Value::Object(42);
        assert_eq!(obj.type_of(), "object");
        assert!(obj.to_boolean()); // Objects are truthy
        assert_eq!(obj.to_string(), "[object Object]");
    }

    #[test]
    fn test_value_function_named() {
        let func = make_test_function(Some("myFunc"));
        let v = Value::Function(Arc::new(Callable::Function(func)));

        assert!(v.is_function());
        assert_eq!(v.type_of(), "function");
        assert!(v.to_boolean());
        assert_eq!(v.to_string(), "[Function: myFunc]");
    }

    #[test]
    fn test_value_function_anonymous() {
        let func = make_test_function(None);
        let v = Value::Function(Arc::new(Callable::Function(func)));

        assert!(v.is_function());
        assert_eq!(v.to_string(), "[Function (anonymous)]");
    }

    #[test]
    fn test_value_function_native() {
        let native = Callable::Native {
            name: "print".to_string(),
            arity: 1,
            func: native_test_fn,
        };
        let v = Value::Function(Arc::new(native));

        assert!(v.is_function());
        assert_eq!(v.to_string(), "[Function: print (native)]");
    }

    #[test]
    fn test_value_default() {
        let v = Value::default();
        assert!(v.is_undefined());
    }

    #[test]
    fn test_value_equality_undefined() {
        assert_eq!(Value::Undefined, Value::Undefined);
        assert_ne!(Value::Undefined, Value::Null);
    }

    #[test]
    fn test_value_equality_null() {
        assert_eq!(Value::Null, Value::Null);
        assert_ne!(Value::Null, Value::Undefined);
    }

    #[test]
    fn test_value_equality_boolean() {
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_eq!(Value::Boolean(false), Value::Boolean(false));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
    }

    #[test]
    fn test_value_equality_number() {
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_ne!(Value::Number(42.0), Value::Number(43.0));

        // NaN is not equal to itself
        assert_ne!(Value::Number(f64::NAN), Value::Number(f64::NAN));
    }

    #[test]
    fn test_value_equality_string() {
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_ne!(
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        );
    }

    #[test]
    fn test_value_equality_symbol() {
        assert_eq!(Value::Symbol(1), Value::Symbol(1));
        assert_ne!(Value::Symbol(1), Value::Symbol(2));
    }

    #[test]
    fn test_value_equality_bigint() {
        assert_eq!(
            Value::BigInt("123".to_string()),
            Value::BigInt("123".to_string())
        );
        assert_ne!(
            Value::BigInt("123".to_string()),
            Value::BigInt("456".to_string())
        );
    }

    #[test]
    fn test_value_equality_object() {
        assert_eq!(Value::Object(1), Value::Object(1));
        assert_ne!(Value::Object(1), Value::Object(2));
    }

    #[test]
    fn test_value_equality_function() {
        let func1 = Arc::new(Callable::Function(make_test_function(None)));
        let func2 = func1.clone();
        let func3 = Arc::new(Callable::Function(make_test_function(None)));

        // Same Arc pointer
        assert_eq!(Value::Function(func1.clone()), Value::Function(func2));
        // Different Arc pointers
        assert_ne!(Value::Function(func1), Value::Function(func3));
    }

    #[test]
    fn test_value_equality_different_types() {
        assert_ne!(Value::Undefined, Value::Boolean(false));
        assert_ne!(Value::Null, Value::Number(0.0));
        assert_ne!(Value::Boolean(true), Value::Number(1.0));
        assert_ne!(Value::String("42".to_string()), Value::Number(42.0));
    }

    #[test]
    fn test_value_debug() {
        // Test that Debug is implemented
        let v = Value::Number(42.0);
        let debug_str = format!("{:?}", v);
        assert!(debug_str.contains("Number"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_value_clone() {
        let original = Value::String("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
