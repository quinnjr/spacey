//! Error built-in objects (ES3 Section 15.11).
//!
//! Provides Error constructor and native error types:
//! - Error
//! - EvalError
//! - RangeError
//! - ReferenceError
//! - SyntaxError
//! - TypeError
//! - URIError

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

/// Error type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ErrorKind {
    Error,
    EvalError,
    RangeError,
    ReferenceError,
    SyntaxError,
    TypeError,
    URIError,
}

#[allow(missing_docs)]
impl ErrorKind {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorKind::Error => "Error",
            ErrorKind::EvalError => "EvalError",
            ErrorKind::RangeError => "RangeError",
            ErrorKind::ReferenceError => "ReferenceError",
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::URIError => "URIError",
        }
    }
}

/// JavaScript Error object representation.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct JsError {
    pub kind: ErrorKind,
    pub message: String,
    pub stack: Option<String>,
}

#[allow(missing_docs)]
impl JsError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            stack: None,
        }
    }

    pub fn with_stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    pub fn name(&self) -> &'static str {
        self.kind.name()
    }

    /// Format error as string (name: message)
    pub fn to_js_string(&self) -> String {
        if self.message.is_empty() {
            self.kind.name().to_string()
        } else {
            format!("{}: {}", self.kind.name(), self.message)
        }
    }
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_js_string())
    }
}

impl std::error::Error for JsError {}

// ============================================================================
// Error Constructors (ES3 Section 15.11.1-2)
// ============================================================================

/// Error(message) constructor - creates a generic Error.
pub fn error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::Error, args)
}

/// EvalError(message) constructor.
pub fn eval_error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::EvalError, args)
}

/// RangeError(message) constructor.
pub fn range_error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::RangeError, args)
}

/// ReferenceError(message) constructor.
pub fn reference_error_constructor(
    _frame: &mut CallFrame,
    args: &[Value],
) -> Result<Value, String> {
    create_error(ErrorKind::ReferenceError, args)
}

/// SyntaxError(message) constructor.
pub fn syntax_error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::SyntaxError, args)
}

/// TypeError(message) constructor.
pub fn type_error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::TypeError, args)
}

/// URIError(message) constructor.
pub fn uri_error_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    create_error(ErrorKind::URIError, args)
}

/// Helper to create error objects.
/// Returns a NativeObject with name and message properties.
fn create_error(kind: ErrorKind, args: &[Value]) -> Result<Value, String> {
    let message = args
        .first()
        .filter(|v| !matches!(v, Value::Undefined))
        .map(|v| v.to_js_string())
        .unwrap_or_default();

    // Create an error object with name and message properties
    let mut error_obj = std::collections::HashMap::new();
    error_obj.insert("name".to_string(), Value::String(kind.name().to_string()));
    error_obj.insert("message".to_string(), Value::String(message.clone()));
    error_obj.insert("__type__".to_string(), Value::String("Error".to_string()));
    error_obj.insert("__error_kind__".to_string(), Value::Number(kind as i32 as f64));

    // toString for error objects
    let to_string_result = if message.is_empty() {
        kind.name().to_string()
    } else {
        format!("{}: {}", kind.name(), message)
    };
    error_obj.insert("__toString__".to_string(), Value::String(to_string_result));

    Ok(Value::NativeObject(error_obj))
}

// ============================================================================
// Error.prototype Methods (ES3 Section 15.11.4)
// ============================================================================

/// Error.prototype.toString() - Returns "name: message".
pub fn error_to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // In real impl, would get name and message from error object
    let name = args
        .get(1)
        .map(|v| v.to_js_string())
        .unwrap_or_else(|| "Error".to_string());
    let message = args.get(2).map(|v| v.to_js_string()).unwrap_or_default();

    let result = if message.is_empty() {
        name
    } else {
        format!("{}: {}", name, message)
    };

    Ok(Value::String(result))
}

// ============================================================================
// Helper Functions for Creating Errors in Rust
// ============================================================================

/// Create a TypeError with the given message.
pub fn type_error(message: impl Into<String>) -> JsError {
    JsError::new(ErrorKind::TypeError, message)
}

/// Create a ReferenceError with the given message.
pub fn reference_error(message: impl Into<String>) -> JsError {
    JsError::new(ErrorKind::ReferenceError, message)
}

/// Create a SyntaxError with the given message.
pub fn syntax_error(message: impl Into<String>) -> JsError {
    JsError::new(ErrorKind::SyntaxError, message)
}

/// Create a RangeError with the given message.
pub fn range_error(message: impl Into<String>) -> JsError {
    JsError::new(ErrorKind::RangeError, message)
}

/// Create a generic Error with the given message.
pub fn error(message: impl Into<String>) -> JsError {
    JsError::new(ErrorKind::Error, message)
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
    fn test_error_kind_name() {
        assert_eq!(ErrorKind::Error.name(), "Error");
        assert_eq!(ErrorKind::TypeError.name(), "TypeError");
        assert_eq!(ErrorKind::ReferenceError.name(), "ReferenceError");
        assert_eq!(ErrorKind::SyntaxError.name(), "SyntaxError");
        assert_eq!(ErrorKind::RangeError.name(), "RangeError");
        assert_eq!(ErrorKind::EvalError.name(), "EvalError");
        assert_eq!(ErrorKind::URIError.name(), "URIError");
    }

    #[test]
    fn test_js_error_new() {
        let err = JsError::new(ErrorKind::TypeError, "not a function");
        assert_eq!(err.kind, ErrorKind::TypeError);
        assert_eq!(err.message, "not a function");
        assert!(err.stack.is_none());
    }

    #[test]
    fn test_js_error_with_stack() {
        let err = JsError::new(ErrorKind::Error, "test").with_stack("at foo\nat bar");
        assert!(err.stack.is_some());
        assert_eq!(err.stack.unwrap(), "at foo\nat bar");
    }

    #[test]
    fn test_js_error_to_string() {
        let err = JsError::new(ErrorKind::TypeError, "not a function");
        assert_eq!(err.to_js_string(), "TypeError: not a function");

        let err_no_msg = JsError::new(ErrorKind::Error, "");
        assert_eq!(err_no_msg.to_js_string(), "Error");
    }

    #[test]
    fn test_js_error_display() {
        let err = JsError::new(ErrorKind::RangeError, "invalid length");
        assert_eq!(format!("{}", err), "RangeError: invalid length");
    }

    #[test]
    fn test_error_constructor() {
        let mut frame = make_frame();
        let result =
            error_constructor(&mut frame, &[Value::String("test error".to_string())]).unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_type_error_constructor() {
        let mut frame = make_frame();
        let result =
            type_error_constructor(&mut frame, &[Value::String("not callable".to_string())])
                .unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_reference_error_constructor() {
        let mut frame = make_frame();
        let result = reference_error_constructor(
            &mut frame,
            &[Value::String("x is not defined".to_string())],
        )
        .unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_syntax_error_constructor() {
        let mut frame = make_frame();
        let result =
            syntax_error_constructor(&mut frame, &[Value::String("unexpected token".to_string())])
                .unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_range_error_constructor() {
        let mut frame = make_frame();
        let result = range_error_constructor(
            &mut frame,
            &[Value::String("invalid array length".to_string())],
        )
        .unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_eval_error_constructor() {
        let mut frame = make_frame();
        let result = eval_error_constructor(&mut frame, &[]).unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_uri_error_constructor() {
        let mut frame = make_frame();
        let result =
            uri_error_constructor(&mut frame, &[Value::String("malformed URI".to_string())])
                .unwrap();
        // Error constructors now return NativeObject
        assert!(matches!(result, Value::NativeObject(_)));
    }

    #[test]
    fn test_error_to_string() {
        let mut frame = make_frame();
        let result = error_to_string(
            &mut frame,
            &[
                Value::Undefined,
                Value::String("Error".to_string()),
                Value::String("something went wrong".to_string()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "Error: something went wrong"));
    }

    #[test]
    fn test_error_to_string_no_message() {
        let mut frame = make_frame();
        let result = error_to_string(
            &mut frame,
            &[Value::Undefined, Value::String("TypeError".to_string())],
        )
        .unwrap();
        assert!(matches!(result, Value::String(s) if s == "TypeError"));
    }

    #[test]
    fn test_helper_functions() {
        let err = type_error("not a function");
        assert_eq!(err.kind, ErrorKind::TypeError);

        let err = reference_error("x is not defined");
        assert_eq!(err.kind, ErrorKind::ReferenceError);

        let err = syntax_error("unexpected token");
        assert_eq!(err.kind, ErrorKind::SyntaxError);

        let err = range_error("invalid length");
        assert_eq!(err.kind, ErrorKind::RangeError);

        let err = error("generic error");
        assert_eq!(err.kind, ErrorKind::Error);
    }

    #[test]
    fn test_error_kind_equality() {
        assert_eq!(ErrorKind::Error, ErrorKind::Error);
        assert_ne!(ErrorKind::Error, ErrorKind::TypeError);
    }

    #[test]
    fn test_js_error_clone() {
        let err = JsError::new(ErrorKind::Error, "test").with_stack("stack");
        let cloned = err.clone();
        assert_eq!(err.kind, cloned.kind);
        assert_eq!(err.message, cloned.message);
        assert_eq!(err.stack, cloned.stack);
    }
}
