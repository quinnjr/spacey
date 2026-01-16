//! JavaScript value representation.

use std::fmt;

/// A JavaScript value.
#[derive(Debug, Clone, PartialEq, Default)]
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
    /// Function reference (index into VM's function table)
    Function(usize),
    /// Native function (builtin ID)
    NativeFunction(u16),
    /// An array of values (used during JSON parsing before VM conversion)
    Array(Vec<Value>),
    /// A parsed object (used during JSON parsing before VM conversion)
    ParsedObject(Vec<(String, Value)>),
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
            | Value::NativeFunction(_)
            | Value::Array(_)
            | Value::ParsedObject(_) => true,
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
            Value::Object(_) | Value::Array(_) | Value::ParsedObject(_) => "object",
            Value::Function(_) | Value::NativeFunction(_) => "function",
        }
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
            Value::Object(_) | Value::ParsedObject(_) => write!(f, "[object Object]"),
            Value::Function(_) | Value::NativeFunction(_) => write!(f, "[Function]"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}
