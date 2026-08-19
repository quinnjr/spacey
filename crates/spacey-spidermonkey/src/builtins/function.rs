//! Function built-in object (ES3 Section 15.3).
//!
//! Provides Function constructor and prototype methods.

use std::sync::Arc;

use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::runtime::function::{CallFrame, Callable, Function};
use crate::runtime::value::Value;

// ============================================================================
// Function Constructor (ES3 Section 15.3.1-2)
// ============================================================================

/// Function() constructor - creates a new function from strings.
///
/// ES3 Section 15.3.1.1
/// Usage: new Function([arg1[, arg2[, ...argN]],] functionBody)
pub fn function_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Parse arguments: all but the last are parameter names, last is the body
    let (params, body) = if args.is_empty() {
        (vec![], String::new())
    } else if args.len() == 1 {
        // Only body provided
        let body = args[0].to_js_string();
        (vec![], body)
    } else {
        // Multiple arguments: params + body
        let params: Vec<String> = args[..args.len() - 1]
            .iter()
            .map(|v| v.to_js_string())
            .collect();
        let body = args[args.len() - 1].to_js_string();
        (params, body)
    };

    // Validate parameter names
    for param in &params {
        // Check for valid identifier (simplified check)
        if param.is_empty()
            || !param
                .chars()
                .next()
                .map(|c| c.is_alphabetic() || c == '_' || c == '$')
                .unwrap_or(false)
        {
            return Err(format!("SyntaxError: invalid parameter name '{}'", param));
        }
    }

    // Create function source code
    let params_str = params.join(", ");
    let source = format!("function anonymous({}) {{ {} }}", params_str, body);

    // Parse the function
    let mut parser = Parser::new(&source);
    let program = parser
        .parse_program()
        .map_err(|e| format!("SyntaxError: {}", e))?;

    // Compile the function
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&program)
        .map_err(|e| format!("SyntaxError: {}", e))?;

    // Create the function object
    let arity = params.len();
    let func = Function::new(Some("anonymous".to_string()), params, bytecode, arity);

    Ok(Value::Function(Arc::new(Callable::Function(func))))
}

// ============================================================================
// Function.prototype Methods (ES3 Section 15.3.4)
// ============================================================================

/// Function.prototype.toString() - Returns function source.
///
/// ES3 Section 15.3.4.2
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Function(callable)) => {
            use crate::runtime::function::Callable;
            match callable.as_ref() {
                Callable::Function(func) => {
                    let name = func.name.as_deref().unwrap_or("");
                    let params = func.params.join(", ");
                    Ok(Value::String(format!(
                        "function {}({}) {{ [native code] }}",
                        name, params
                    )))
                }
                Callable::Native { name, .. } => Ok(Value::String(format!(
                    "function {}() {{ [native code] }}",
                    name
                ))),
            }
        }
        _ => Err("TypeError: Function.prototype.toString requires a function".to_string()),
    }
}

/// Function.prototype.apply(thisArg, argArray) - Calls function with given this and args.
///
/// ES3 Section 15.3.4.3
pub fn apply(frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // First argument should be the function (this in method call)
    let func = args.first().cloned().unwrap_or(Value::Undefined);

    // Second argument is thisArg
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Third argument is argArray
    let arg_array = args.get(2).cloned().unwrap_or(Value::Undefined);

    // Convert argArray to arguments
    let call_args = match &arg_array {
        Value::Null | Value::Undefined => vec![],
        Value::Object(_) => {
            // In full impl, would iterate array-like object
            // For now, return empty
            vec![]
        }
        _ => {
            return Err("TypeError: second argument to apply must be an array".to_string());
        }
    };

    // Call the function
    call_function(frame, &func, &this_arg, &call_args)
}

/// Function.prototype.call(thisArg, arg1, arg2, ...) - Calls function with given this.
///
/// ES3 Section 15.3.4.4
pub fn call(frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // First argument should be the function (this in method call)
    let func = args.first().cloned().unwrap_or(Value::Undefined);

    // Second argument is thisArg
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Remaining arguments are passed to the function
    let call_args: Vec<Value> = args.iter().skip(2).cloned().collect();

    // Call the function
    call_function(frame, &func, &this_arg, &call_args)
}

/// Function.prototype.bind(thisArg, arg1, arg2, ...) - Creates bound function.
///
/// Note: This is ES5, but commonly needed
pub fn bind(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // First argument should be the function
    let func = args.first().cloned().unwrap_or(Value::Undefined);

    if !matches!(func, Value::Function(_)) {
        return Err("TypeError: bind must be called on a function".to_string());
    }

    // In a full implementation, would create a BoundFunction object
    // For now, just return the original function
    Ok(func)
}

// ============================================================================
// Function.prototype Properties
// ============================================================================

/// Function.prototype.length - Returns the expected number of arguments.
pub fn get_length(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Function(callable)) => {
            use crate::runtime::function::Callable;
            let arity = match callable.as_ref() {
                Callable::Function(func) => func.params.len() as i32,
                Callable::Native { arity, .. } => *arity,
            };
            Ok(Value::Number(arity.max(0) as f64))
        }
        _ => Ok(Value::Number(0.0)),
    }
}

/// Function.prototype.name - Returns the function name.
pub fn get_name(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Function(callable)) => {
            use crate::runtime::function::Callable;
            let name = match callable.as_ref() {
                Callable::Function(func) => func.name.clone().unwrap_or_default(),
                Callable::Native { name, .. } => name.clone(),
            };
            Ok(Value::String(name))
        }
        _ => Ok(Value::String(String::new())),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calls a function with the given this value and arguments.
fn call_function(
    frame: &mut CallFrame,
    func: &Value,
    _this_arg: &Value,
    args: &[Value],
) -> Result<Value, String> {
    match func {
        Value::Function(callable) => {
            use crate::runtime::function::Callable;
            match callable.as_ref() {
                Callable::Function(_func) => {
                    // In full impl, would set up call frame with this binding
                    // For now, return undefined
                    Ok(Value::Undefined)
                }
                Callable::Native {
                    func: native_fn, ..
                } => {
                    // Call native function
                    native_fn(frame, args)
                }
            }
        }
        _ => Err("TypeError: not a function".to_string()),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::{Callable, Function};
    use std::sync::Arc;

    fn make_frame() -> CallFrame {
        let func = Function::new(None, vec![], Bytecode::new(), 0);
        CallFrame::new(func, 0)
    }

    fn make_test_function(name: Option<&str>, params: Vec<&str>) -> Value {
        let func = Function::new(
            name.map(|s| s.to_string()),
            params.into_iter().map(|s| s.to_string()).collect(),
            Bytecode::new(),
            0,
        );
        Value::Function(Arc::new(Callable::Function(func)))
    }

    fn make_native_function(name: &str, arity: i32) -> Value {
        Value::Function(Arc::new(Callable::Native {
            name: name.to_string(),
            arity,
            func: |_, _| Ok(Value::Undefined),
        }))
    }

    #[test]
    fn test_to_string_user_function() {
        let mut frame = make_frame();
        let func = make_test_function(Some("myFunc"), vec!["a", "b"]);
        let result = to_string(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::String(s) if s.contains("myFunc") && s.contains("a, b")));
    }

    #[test]
    fn test_to_string_anonymous_function() {
        let mut frame = make_frame();
        let func = make_test_function(None, vec![]);
        let result = to_string(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::String(s) if s.contains("function")));
    }

    #[test]
    fn test_to_string_native_function() {
        let mut frame = make_frame();
        let func = make_native_function("nativeFunc", 2);
        let result = to_string(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::String(s) if s.contains("nativeFunc")));
    }

    #[test]
    fn test_to_string_not_function() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_length_user_function() {
        let mut frame = make_frame();
        let func = make_test_function(Some("f"), vec!["a", "b", "c"]);
        let result = get_length(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_get_length_native_function() {
        let mut frame = make_frame();
        let func = make_native_function("f", 2);
        let result = get_length(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_get_length_variadic() {
        let mut frame = make_frame();
        let func = make_native_function("f", -1);
        let result = get_length(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_get_name() {
        let mut frame = make_frame();
        let func = make_test_function(Some("myFunction"), vec![]);
        let result = get_name(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "myFunction"));
    }

    #[test]
    fn test_get_name_anonymous() {
        let mut frame = make_frame();
        let func = make_test_function(None, vec![]);
        let result = get_name(&mut frame, &[func]).unwrap();
        assert!(matches!(result, Value::String(s) if s.is_empty()));
    }

    #[test]
    fn test_call_native() {
        let mut frame = make_frame();

        // Create a native function that returns a specific value
        let native = Value::Function(Arc::new(Callable::Native {
            name: "testFn".to_string(),
            arity: 1,
            func: |_, args| {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(Value::Number(n * 2.0))
            },
        }));

        // Call with arguments
        let result = call(&mut frame, &[native, Value::Null, Value::Number(21.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_call_not_function() {
        let mut frame = make_frame();
        let result = call(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_empty_args() {
        let mut frame = make_frame();
        let native = make_native_function("f", 0);
        let result = apply(&mut frame, &[native, Value::Null, Value::Null]).unwrap();
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_bind_returns_function() {
        let mut frame = make_frame();
        let func = make_test_function(Some("f"), vec![]);
        let result = bind(&mut frame, &[func.clone()]).unwrap();
        assert!(matches!(result, Value::Function(_)));
    }

    #[test]
    fn test_bind_not_function() {
        let mut frame = make_frame();
        let result = bind(&mut frame, &[Value::Number(42.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_constructor() {
        let mut frame = make_frame();
        // Empty function should work
        let result = function_constructor(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Function(_)));
    }

    #[test]
    fn test_function_constructor_with_body() {
        let mut frame = make_frame();
        // Function with body - note: body needs proper JS syntax
        let result = function_constructor(&mut frame, &[Value::String("return 42;".to_string())]);
        assert!(result.is_ok(), "Function constructor failed: {:?}", result);
        assert!(matches!(result.unwrap(), Value::Function(_)));
    }

    #[test]
    fn test_function_constructor_with_params() {
        let mut frame = make_frame();
        // Function with params and body
        let result = function_constructor(
            &mut frame,
            &[
                Value::String("x".to_string()),
                Value::String("y".to_string()),
                Value::String("return x + y;".to_string()),
            ],
        );
        assert!(result.is_ok(), "Function constructor failed: {:?}", result);
        assert!(matches!(result.unwrap(), Value::Function(_)));
    }
}
