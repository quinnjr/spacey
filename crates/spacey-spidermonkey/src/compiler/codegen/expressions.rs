//! Expression compilation documentation.
//!
//! This module documents the expression compilation logic in `mod.rs`.
//! Expressions are compiled to bytecode that produces values on the stack.
//!
//! ## Expression Compilation Overview
//!
//! | Expression | Key Operations | Stack Effect |
//! |------------|----------------|--------------|
//! | Literal | `Constant` | Push value |
//! | Identifier | `GetLocal`/`GetGlobal` | Push value |
//! | Binary | `Add`/`Sub`/etc | Pop 2, push 1 |
//! | Unary | `Negate`/`Not`/etc | Pop 1, push 1 |
//! | Assignment | `SetLocal`/`SetGlobal` | May push value |
//! | Call | `Call` | Pop N+1, push result |
//! | Member | `GetProperty` | Pop 1, push 1 |
//! | Array | `CreateArray` | Pop N, push 1 |
//! | Object | `CreateObject` | Pop N*2, push 1 |
//! | Function | `CreateFunction` | Push closure |
//! | Conditional | `JumpIfFalse`, `Jump` | Pop 1, push 1 |
//!
//! ## Stack Machine Model
//!
//! The VM uses a stack-based execution model:
//!
//! ```text
//! Expression: 1 + 2 * 3
//!
//! Bytecode:
//!   Constant 1      ; stack: [1]
//!   Constant 2      ; stack: [1, 2]
//!   Constant 3      ; stack: [1, 2, 3]
//!   Multiply        ; stack: [1, 6]
//!   Add             ; stack: [7]
//! ```
//!
//! ## Binary Operators
//!
//! Binary operators compile both operands left-to-right, then apply the operation:
//!
//! | Operator | OpCode | Notes |
//! |----------|--------|-------|
//! | `+` | `Add` | String concatenation if either is string |
//! | `-` | `Sub` | |
//! | `*` | `Mul` | |
//! | `/` | `Div` | |
//! | `%` | `Mod` | |
//! | `==` | `Equal` | Abstract equality (type coercion) |
//! | `===` | `StrictEqual` | Strict equality |
//! | `!=` | `NotEqual` | |
//! | `!==` | `StrictNotEqual` | |
//! | `<` | `LessThan` | |
//! | `>` | `GreaterThan` | |
//! | `<=` | `LessThanEqual` | |
//! | `>=` | `GreaterThanEqual` | |
//! | `&` | `BitwiseAnd` | |
//! | `\|` | `BitwiseOr` | |
//! | `^` | `BitwiseXor` | |
//! | `<<` | `ShiftLeft` | |
//! | `>>` | `ShiftRight` | |
//! | `>>>` | `UnsignedShiftRight` | |
//! | `in` | `In` | Property existence |
//! | `instanceof` | `InstanceOf` | Prototype chain check |
//!
//! ## Logical Operators (Short-Circuit)
//!
//! `&&` and `||` use short-circuit evaluation:
//!
//! ```text
//! Expression: a && b
//!
//! Bytecode:
//!   [compile a]
//!   Dup              ; stack: [a, a]
//!   JumpIfFalse end  ; if falsy, skip b
//!   Pop              ; discard first a
//!   [compile b]
//! end:
//! ```
//!
//! ## Unary Operators
//!
//! | Operator | OpCode | Notes |
//! |----------|--------|-------|
//! | `-` | `Negate` | |
//! | `+` | `ToNumber` | Numeric coercion |
//! | `!` | `Not` | Boolean negation |
//! | `~` | `BitwiseNot` | |
//! | `typeof` | `TypeOf` | Returns type string |
//! | `void` | `Void` | Returns undefined |
//! | `delete` | `Delete` | Property deletion |
//!
//! ## Update Expressions
//!
//! `++` and `--` operators have prefix and postfix variants:
//!
//! ```text
//! // Prefix: ++x
//! GetLocal x
//! Constant 1
//! Add
//! Dup
//! SetLocal x
//!
//! // Postfix: x++
//! GetLocal x
//! Dup
//! Constant 1
//! Add
//! SetLocal x
//! Pop  ; leave original value
//! ```
//!
//! ## Member Access
//!
//! Computed (`obj[prop]`) and non-computed (`obj.prop`) member access:
//!
//! ```text
//! // obj.prop
//! [compile obj]
//! GetProperty "prop"
//!
//! // obj[expr]
//! [compile obj]
//! [compile expr]
//! GetPropertyComputed
//! ```
//!
//! ## Function Calls
//!
//! ```text
//! // func(a, b, c)
//! [compile func]     ; stack: [func]
//! [compile a]        ; stack: [func, a]
//! [compile b]        ; stack: [func, a, b]
//! [compile c]        ; stack: [func, a, b, c]
//! Call 3             ; stack: [result]
//! ```
//!
//! ## Closures
//!
//! Function expressions capture their enclosing scope:
//!
//! ```text
//! function outer() {
//!     var x = 10;
//!     return function inner() {
//!         return x;  // captures x from outer
//!     };
//! }
//! ```
//!
//! The compiler tracks captured variables in `closure_env`.

// This module serves as documentation. The actual implementation is in mod.rs.

#[cfg(test)]
mod tests {
    use crate::compiler::Compiler;
    use crate::parser::Parser;

    fn compile(src: &str) -> crate::compiler::Bytecode {
        let mut parser = Parser::new(src);
        let program = parser.parse_program().expect("Should parse");
        let mut compiler = Compiler::new();
        compiler.compile(&program).expect("Should compile")
    }

    #[test]
    fn test_compile_number() {
        let bc = compile("42;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_string() {
        let bc = compile("'hello';");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_boolean() {
        let bc = compile("true;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_null() {
        let bc = compile("null;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_identifier() {
        let bc = compile("x;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_binary_add() {
        let bc = compile("1 + 2;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_binary_multiply() {
        let bc = compile("3 * 4;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_binary_compare() {
        let bc = compile("a < b;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_logical_and() {
        let bc = compile("a && b;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_logical_or() {
        let bc = compile("a || b;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_unary_not() {
        let bc = compile("!x;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_unary_negate() {
        let bc = compile("-x;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_typeof() {
        let bc = compile("typeof x;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_assignment() {
        let bc = compile("x = 5;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_call() {
        let bc = compile("foo();");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_call_with_args() {
        let bc = compile("foo(1, 2, 3);");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_member_dot() {
        let bc = compile("obj.prop;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_member_bracket() {
        let bc = compile("arr[0];");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_array() {
        let bc = compile("[1, 2, 3];");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_object() {
        // Note: need parentheses to disambiguate from block statement
        let bc = compile("var o = { a: 1, b: 2 };");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_function_expr() {
        let bc = compile("(function(x) { return x; });");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_arrow() {
        let bc = compile("(x) => x * 2;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_conditional() {
        let bc = compile("x ? 1 : 2;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_new() {
        let bc = compile("new Foo();");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_update_prefix() {
        let bc = compile("++x;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_update_postfix() {
        let bc = compile("x++;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_delete() {
        let bc = compile("delete obj.prop;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_void() {
        let bc = compile("void 0;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_in() {
        let bc = compile("'x' in obj;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_instanceof() {
        let bc = compile("x instanceof Array;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_this() {
        let bc = compile("this;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_sequence() {
        let bc = compile("(a, b, c);");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_chained_member() {
        let bc = compile("a.b.c;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_method_call() {
        let bc = compile("obj.method();");
        assert!(!bc.instructions.is_empty());
    }
}
