// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! # spacey-spidermonkey
//!
//! A JavaScript engine inspired by Mozilla's SpiderMonkey, implemented in Rust.
//!
//! ## Overview
//!
//! This crate provides a complete JavaScript execution environment including:
//! - Lexer and parser for ECMAScript 2024+
//! - Bytecode compiler and interpreter
//! - Garbage-collected runtime
//! - Built-in objects and standard library
//! - Optional JIT compilation
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use spacey_spidermonkey::{Engine, Value};
//!
//! let mut engine = Engine::new();
//! let result = engine.eval("1 + 2")?;
//! assert_eq!(result, Value::Number(3.0));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules - to be implemented
pub mod ast;
pub mod builtins;
pub mod compiler;
pub mod gc;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod vm;

// Re-exports for convenience
pub use runtime::context::Context;
pub use runtime::value::Value;

/// The main JavaScript engine instance.
///
/// Encapsulates the entire JavaScript execution environment including
/// the heap, global object, and execution state.
pub struct Engine {
    /// The VM instance for executing bytecode
    vm: vm::VM,
}

impl Engine {
    /// Creates a new JavaScript engine instance with default configuration.
    pub fn new() -> Self {
        Self { vm: vm::VM::new() }
    }

    /// Evaluates JavaScript source code and returns the result.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript source code to evaluate
    ///
    /// # Returns
    ///
    /// The result of evaluating the expression, or an error if parsing
    /// or execution fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new();
    /// let result = engine.eval("2 + 2")?;
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        // 1. Parse source into AST
        let mut parser = parser::Parser::new(source);
        let program = parser.parse_program()?;

        // 2. Compile AST to bytecode
        let mut compiler = compiler::Compiler::new();
        let bytecode = compiler.compile(&program)?;

        // 3. Execute bytecode in VM
        self.vm.execute(&bytecode)
    }

    /// Evaluates JavaScript source code from a file.
    pub fn eval_file(&mut self, path: &std::path::Path) -> Result<Value, Error> {
        let source = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;
        self.eval(&source)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during JavaScript execution.
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax error during parsing
    SyntaxError(String),
    /// Type error during execution
    TypeError(String),
    /// Reference error (undefined variable)
    ReferenceError(String),
    /// Range error (out of bounds, etc.)
    RangeError(String),
    /// Internal engine error
    InternalError(String),
    /// I/O error
    Io(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            Error::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Error::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            Error::RangeError(msg) => write!(f, "RangeError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Error::Io(msg) => write!(f, "IOError: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = Engine::new();
    }

    #[test]
    fn test_eval_number() {
        let mut engine = Engine::new();
        let result = engine.eval("42;").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut engine = Engine::new();
        let result = engine.eval("2 + 3 * 4;").unwrap();
        assert_eq!(result, Value::Number(14.0));
    }

    #[test]
    fn test_eval_string() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\";").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_string_concat() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\" + \" world\";").unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_eval_boolean() {
        let mut engine = Engine::new();
        let result = engine.eval("true;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_comparison() {
        let mut engine = Engine::new();
        let result = engine.eval("5 > 3;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_conditional() {
        let mut engine = Engine::new();
        let result = engine.eval("true ? 1 : 2;").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eval_logical_and() {
        let mut engine = Engine::new();
        let result = engine.eval("true && false;").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_logical_or() {
        let mut engine = Engine::new();
        let result = engine.eval("false || true;").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_variable() {
        let mut engine = Engine::new();
        let result = engine.eval("var x = 10; x;").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_eval_array_literal() {
        let mut engine = Engine::new();
        let result = engine.eval("[1, 2, 3];").unwrap();
        assert!(matches!(result, Value::Object(_))); // Arrays are objects in JS
    }

    #[test]
    fn test_eval_function_return_constant() {
        let mut engine = Engine::new();
        // Simplest possible function test
        let result = engine.eval("function f() { return 42; } f();").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_function_declaration() {
        let mut engine = Engine::new();
        let result = engine
            .eval("function add(a, b) { return a + b; } add(2, 3);")
            .unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_function_expression() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var mul = function(a, b) { return a * b; }; mul(3, 4);")
            .unwrap();
        assert_eq!(result, Value::Number(12.0));
    }

    #[test]
    fn test_eval_arrow_function() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var square = (x) => x * x; square(5);")
            .unwrap();
        assert_eq!(result, Value::Number(25.0));
    }

    #[test]
    fn test_eval_arrow_function_expression_body() {
        let mut engine = Engine::new();
        let result = engine.eval("var double = x => x * 2; double(7);").unwrap();
        assert_eq!(result, Value::Number(14.0));
    }

    #[test]
    fn test_eval_nested_function_calls() {
        let mut engine = Engine::new();
        let result = engine.eval("function outer(x) { function inner(y) { return y + 1; } return inner(x) * 2; } outer(5);").unwrap();
        assert_eq!(result, Value::Number(12.0));
    }

    #[test]
    fn test_eval_typeof_function() {
        let mut engine = Engine::new();
        let result = engine.eval("function f() {} typeof f;").unwrap();
        assert_eq!(result, Value::String("function".to_string()));
    }

    // Built-in function tests
    #[test]
    fn test_eval_parse_int() {
        let mut engine = Engine::new();
        let result = engine.eval("parseInt('42');").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_parse_int_radix() {
        let mut engine = Engine::new();
        let result = engine.eval("parseInt('ff', 16);").unwrap();
        assert_eq!(result, Value::Number(255.0));
    }

    #[test]
    fn test_eval_parse_float() {
        let mut engine = Engine::new();
        let result = engine.eval("parseFloat('3.14');").unwrap();
        assert_eq!(result, Value::Number(3.14));
    }

    #[test]
    fn test_eval_is_nan() {
        let mut engine = Engine::new();
        let result = engine.eval("isNaN(NaN);").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_is_finite() {
        let mut engine = Engine::new();
        let result = engine.eval("isFinite(42);").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_is_finite_infinity() {
        let mut engine = Engine::new();
        let result = engine.eval("isFinite(Infinity);").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_math_floor() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.floor(4.7);").unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_eval_math_ceil() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.ceil(4.2);").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_math_round() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.round(4.5);").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_math_abs() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.abs(-5);").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_math_max() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.max(1, 5, 3);").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_math_min() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.min(1, 5, 3);").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eval_math_pow() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.pow(2, 3);").unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_eval_math_sqrt() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.sqrt(16);").unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_eval_math_pi() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.PI;").unwrap();
        assert_eq!(result, Value::Number(std::f64::consts::PI));
    }

    #[test]
    fn test_eval_math_e() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.E;").unwrap();
        assert_eq!(result, Value::Number(std::f64::consts::E));
    }

    #[test]
    fn test_eval_math_sin() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.sin(0);").unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_eval_math_cos() {
        let mut engine = Engine::new();
        let result = engine.eval("Math.cos(0);").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    // String prototype method tests
    #[test]
    fn test_eval_string_to_upper_case() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".toUpperCase();").unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_eval_string_to_lower_case() {
        let mut engine = Engine::new();
        let result = engine.eval("\"HELLO\".toLowerCase();").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_string_char_at() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".charAt(1);").unwrap();
        assert_eq!(result, Value::String("e".to_string()));
    }

    #[test]
    fn test_eval_string_index_of() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".indexOf(\"l\");").unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_eval_string_slice() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".slice(1, 4);").unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn test_eval_string_trim() {
        let mut engine = Engine::new();
        let result = engine.eval("\"  hello  \".trim();").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_string_length() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".length;").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_string_repeat() {
        let mut engine = Engine::new();
        let result = engine.eval("\"ab\".repeat(3);").unwrap();
        assert_eq!(result, Value::String("ababab".to_string()));
    }

    #[test]
    fn test_eval_string_starts_with() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".startsWith(\"hel\");").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_string_includes() {
        let mut engine = Engine::new();
        let result = engine.eval("\"hello\".includes(\"ell\");").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    // Number prototype method tests
    #[test]
    fn test_eval_number_to_fixed() {
        let mut engine = Engine::new();
        let result = engine.eval("(3.14159).toFixed(2);").unwrap();
        assert_eq!(result, Value::String("3.14".to_string()));
    }

    // Array prototype method tests
    #[test]
    fn test_eval_array_push() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var arr = [1, 2]; arr.push(3); arr.length;")
            .unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_eval_array_pop() {
        let mut engine = Engine::new();
        let result = engine.eval("var arr = [1, 2, 3]; arr.pop();").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_eval_array_join() {
        let mut engine = Engine::new();
        let result = engine.eval("[1, 2, 3].join('-');").unwrap();
        assert_eq!(result, Value::String("1-2-3".to_string()));
    }

    #[test]
    fn test_eval_array_index_of() {
        let mut engine = Engine::new();
        let result = engine.eval("[1, 2, 3, 2].indexOf(2);").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eval_array_reverse() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var arr = [1, 2, 3]; arr.reverse(); arr.join(',');")
            .unwrap();
        assert_eq!(result, Value::String("3,2,1".to_string()));
    }

    // ES5 Array methods
    #[test]
    fn test_eval_array_foreach() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var sum = 0; [1, 2, 3].forEach(function(x) { sum = sum + x; }); sum;")
            .unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_eval_array_map() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 2, 3].map(function(x) { return x * 2; }).join(',');")
            .unwrap();
        assert_eq!(result, Value::String("2,4,6".to_string()));
    }

    #[test]
    fn test_eval_array_filter() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 2, 3, 4, 5].filter(function(x) { return x > 2; }).join(',');")
            .unwrap();
        assert_eq!(result, Value::String("3,4,5".to_string()));
    }

    #[test]
    fn test_eval_array_every() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[2, 4, 6].every(function(x) { return x % 2 === 0; });")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_array_every_false() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[2, 3, 6].every(function(x) { return x % 2 === 0; });")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_array_some() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 3, 5].some(function(x) { return x % 2 === 0; });")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_eval_array_some_true() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 2, 5].some(function(x) { return x % 2 === 0; });")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_array_reduce() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 2, 3, 4].reduce(function(acc, x) { return acc + x; }, 0);")
            .unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_eval_array_reduce_no_initial() {
        let mut engine = Engine::new();
        let result = engine
            .eval("[1, 2, 3, 4].reduce(function(acc, x) { return acc + x; });")
            .unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_eval_array_reduce_right() {
        let mut engine = Engine::new();
        let result = engine
            .eval("['a', 'b', 'c'].reduceRight(function(acc, x) { return acc + x; }, '');")
            .unwrap();
        assert_eq!(result, Value::String("cba".to_string()));
    }

    #[test]
    fn test_eval_array_is_array() {
        let mut engine = Engine::new();
        let result = engine.eval("Array.isArray([1, 2, 3]);").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_array_is_array_false() {
        let mut engine = Engine::new();
        let result = engine.eval("Array.isArray('hello');").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    // Object static method tests
    #[test]
    fn test_eval_object_keys() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var obj = {a: 1, b: 2}; Object.keys(obj).length;")
            .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_eval_object_values() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var obj = {a: 1, b: 2}; Object.values(obj).length;")
            .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_eval_object_entries() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var obj = {a: 1}; Object.entries(obj).length;")
            .unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eval_object_create() {
        let mut engine = Engine::new();
        let result = engine
            .eval("var obj = Object.create(null); typeof obj;")
            .unwrap();
        assert_eq!(result, Value::String("object".to_string()));
    }

    // JSON.parse tests
    #[test]
    fn test_json_parse_null() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.parse('null');").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_json_parse_boolean() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.eval("JSON.parse('true');").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            engine.eval("JSON.parse('false');").unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_json_parse_number() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.eval("JSON.parse('42');").unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            engine.eval("JSON.parse('-3.14');").unwrap(),
            Value::Number(-3.14)
        );
    }

    #[test]
    fn test_json_parse_string() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.eval("JSON.parse('\"hello\"');").unwrap(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn test_json_parse_array() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.parse('[1, 2, 3]').length;").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_json_parse_object() {
        let mut engine = Engine::new();
        let result = engine
            .eval("JSON.parse('{\"a\": 1, \"b\": 2}').a;")
            .unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_json_parse_nested() {
        let mut engine = Engine::new();
        let result = engine
            .eval("JSON.parse('{\"arr\": [1, 2]}').arr.length;")
            .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_json_parse_invalid() {
        let mut engine = Engine::new();
        assert!(engine.eval("JSON.parse('{invalid}');").is_err());
    }

    // JSON.stringify tests
    #[test]
    fn test_json_stringify_null() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.stringify(null);").unwrap();
        assert_eq!(result, Value::String("null".into()));
    }

    #[test]
    fn test_json_stringify_boolean() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.eval("JSON.stringify(true);").unwrap(),
            Value::String("true".into())
        );
    }

    #[test]
    fn test_json_stringify_number() {
        let mut engine = Engine::new();
        assert_eq!(
            engine.eval("JSON.stringify(42);").unwrap(),
            Value::String("42".into())
        );
    }

    #[test]
    fn test_json_stringify_string() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.stringify(\"hello\");").unwrap();
        assert_eq!(result, Value::String("\"hello\"".into()));
    }

    #[test]
    fn test_json_stringify_array() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.stringify([1, 2, 3]);").unwrap();
        assert_eq!(result, Value::String("[1,2,3]".into()));
    }

    #[test]
    fn test_json_stringify_object() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.stringify({a: 1});").unwrap();
        assert_eq!(result, Value::String("{\"a\":1}".into()));
    }

    #[test]
    fn test_json_stringify_undefined() {
        let mut engine = Engine::new();
        let result = engine.eval("JSON.stringify(undefined);").unwrap();
        assert_eq!(result, Value::Undefined);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut engine = Engine::new();
        let result = engine
            .eval("JSON.parse(JSON.stringify({a: 1, b: [2, 3]})).b.length;")
            .unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    // JSON.stringify space parameter tests
    #[test]
    fn test_json_stringify_space_number() {
        let mut engine = Engine::new();
        let result = engine
            .eval(r#"JSON.stringify({a: 1, b: 2}, null, 2)"#)
            .unwrap();
        assert!(result.to_string().contains("\n"));
    }

    #[test]
    fn test_json_stringify_space_string() {
        let mut engine = Engine::new();
        let result = engine
            .eval(r#"JSON.stringify({a: 1}, null, "\t")"#)
            .unwrap();
        assert!(result.to_string().contains("\t"));
    }

    #[test]
    fn test_json_stringify_space_zero() {
        let mut engine = Engine::new();
        let result = engine.eval(r#"JSON.stringify({a: 1}, null, 0)"#).unwrap();
        assert!(!result.to_string().contains("\n"));
    }
}
