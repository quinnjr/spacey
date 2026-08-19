//! Expression parsing utilities and documentation.
//!
//! This module documents the expression parsing logic found in `parser.rs`.
//! Expression parsing uses recursive descent with operator precedence.
//!
//! ## Precedence Table (lowest to highest)
//!
//! | Precedence | Operators | Method |
//! |------------|-----------|--------|
//! | 1 | `,` | (comma expressions) |
//! | 2 | `=` `+=` `-=` etc. | `parse_assignment` |
//! | 3 | `?:` | `parse_conditional` |
//! | 4 | `\|\|` | `parse_logical_or` |
//! | 5 | `&&` | `parse_logical_and` |
//! | 6 | `\|` | `parse_bitwise_or` |
//! | 7 | `^` | `parse_bitwise_xor` |
//! | 8 | `&` | `parse_bitwise_and` |
//! | 9 | `==` `!=` `===` `!==` | `parse_equality` |
//! | 10 | `<` `>` `<=` `>=` `in` `instanceof` | `parse_comparison` |
//! | 11 | `<<` `>>` `>>>` | `parse_shift` |
//! | 12 | `+` `-` | `parse_additive` |
//! | 13 | `*` `/` `%` | `parse_multiplicative` |
//! | 14 | `!` `~` `+` `-` `typeof` `void` `delete` | `parse_unary` |
//! | 15 | `++` `--` (postfix) | `parse_call` |
//! | 16 | `.` `[]` `()` | `parse_call` |
//! | 17 | `new` | `parse_new_expression` |
//! | 18 | primary | `parse_primary` |
//!
//! ## Primary Expressions
//!
//! - Identifiers: `foo`, `bar`
//! - Literals: `42`, `"hello"`, `true`, `null`
//! - Array literals: `[1, 2, 3]`
//! - Object literals: `{ a: 1, b: 2 }`
//! - Function expressions: `function(x) { return x; }`
//! - Arrow functions: `(x) => x * 2`
//! - Parenthesized: `(a + b)`
//! - `this`, `new`
//!
//! ## Grammar
//!
//! ```text
//! Expression :
//!     AssignmentExpression
//!     Expression , AssignmentExpression
//!
//! AssignmentExpression :
//!     ConditionalExpression
//!     LeftHandSideExpression = AssignmentExpression
//!     LeftHandSideExpression AssignmentOperator AssignmentExpression
//!
//! ConditionalExpression :
//!     LogicalORExpression
//!     LogicalORExpression ? AssignmentExpression : AssignmentExpression
//! ```

// This module serves as documentation. The actual implementation is in parser.rs.
// Future refactoring could move expression parsing methods here.

#[cfg(test)]
mod tests {
    use crate::ast::Expression;
    use crate::parser::Parser;

    fn parse_expr(src: &str) -> Expression {
        let source = format!("{};", src);
        let mut parser = Parser::new(&source);
        parser.parse_expression().expect("Should parse")
    }

    #[test]
    fn test_parse_number() {
        let expr = parse_expr("42");
        assert!(matches!(expr, Expression::Literal(_)));
    }

    #[test]
    fn test_parse_string() {
        let expr = parse_expr("'hello'");
        assert!(matches!(expr, Expression::Literal(_)));
    }

    #[test]
    fn test_parse_boolean() {
        let expr = parse_expr("true");
        assert!(matches!(expr, Expression::Literal(_)));
    }

    #[test]
    fn test_parse_null() {
        let expr = parse_expr("null");
        assert!(matches!(expr, Expression::Literal(_)));
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expr("foo");
        assert!(matches!(expr, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_this() {
        let expr = parse_expr("this");
        assert!(matches!(expr, Expression::This));
    }

    #[test]
    fn test_parse_binary_add() {
        let expr = parse_expr("1 + 2");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_binary_multiply() {
        let expr = parse_expr("3 * 4");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let expr = parse_expr("1 + 2 * 3");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_conditional() {
        let expr = parse_expr("x ? 1 : 2");
        assert!(matches!(expr, Expression::Conditional(_)));
    }

    #[test]
    fn test_parse_assignment() {
        let expr = parse_expr("x = 5");
        assert!(matches!(expr, Expression::Assignment(_)));
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = parse_expr("a || b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_logical_and() {
        let expr = parse_expr("a && b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_comparison() {
        let expr = parse_expr("a < b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_equality() {
        let expr = parse_expr("a === b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_unary_not() {
        let expr = parse_expr("!x");
        assert!(matches!(expr, Expression::Unary(_)));
    }

    #[test]
    fn test_parse_unary_negative() {
        let expr = parse_expr("-x");
        assert!(matches!(expr, Expression::Unary(_)));
    }

    #[test]
    fn test_parse_typeof() {
        let expr = parse_expr("typeof x");
        assert!(matches!(expr, Expression::Unary(_)));
    }

    #[test]
    fn test_parse_call() {
        let expr = parse_expr("foo()");
        assert!(matches!(expr, Expression::Call(_)));
    }

    #[test]
    fn test_parse_call_with_args() {
        let expr = parse_expr("foo(1, 2, 3)");
        assert!(matches!(expr, Expression::Call(_)));
    }

    #[test]
    fn test_parse_member_dot() {
        let expr = parse_expr("obj.prop");
        assert!(matches!(expr, Expression::Member(_)));
    }

    #[test]
    fn test_parse_member_bracket() {
        let expr = parse_expr("arr[0]");
        assert!(matches!(expr, Expression::Member(_)));
    }

    #[test]
    fn test_parse_new() {
        let expr = parse_expr("new Foo()");
        assert!(matches!(expr, Expression::New(_)));
    }

    #[test]
    fn test_parse_new_with_args() {
        let expr = parse_expr("new Date(2024, 1, 1)");
        assert!(matches!(expr, Expression::New(_)));
    }

    #[test]
    fn test_parse_array_literal() {
        let expr = parse_expr("[1, 2, 3]");
        assert!(matches!(expr, Expression::Array(_)));
    }

    #[test]
    fn test_parse_object_literal() {
        let expr = parse_expr("{ a: 1, b: 2 }");
        assert!(matches!(expr, Expression::Object(_)));
    }

    #[test]
    fn test_parse_function_expression() {
        let expr = parse_expr("function(x) { return x; }");
        assert!(matches!(expr, Expression::Function(_)));
    }

    #[test]
    fn test_parse_arrow_function() {
        let expr = parse_expr("(x) => x * 2");
        assert!(matches!(expr, Expression::Arrow(_)));
    }

    #[test]
    fn test_parse_arrow_function_no_parens() {
        let expr = parse_expr("x => x * 2");
        assert!(matches!(expr, Expression::Arrow(_)));
    }

    #[test]
    fn test_parse_parenthesized() {
        let expr = parse_expr("(1 + 2) * 3");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_update_prefix() {
        let expr = parse_expr("++x");
        assert!(matches!(expr, Expression::Update(_)));
    }

    #[test]
    fn test_parse_update_postfix() {
        let expr = parse_expr("x++");
        assert!(matches!(expr, Expression::Update(_)));
    }

    #[test]
    fn test_parse_in_operator() {
        let expr = parse_expr("'x' in obj");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_instanceof() {
        let expr = parse_expr("x instanceof Array");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_bitwise_or() {
        let expr = parse_expr("a | b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_bitwise_and() {
        let expr = parse_expr("a & b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_bitwise_xor() {
        let expr = parse_expr("a ^ b");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_shift_left() {
        let expr = parse_expr("a << 2");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_shift_right() {
        let expr = parse_expr("a >> 2");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_chained_calls() {
        let expr = parse_expr("foo().bar().baz()");
        assert!(matches!(expr, Expression::Call(_)));
    }

    #[test]
    fn test_parse_method_call() {
        let expr = parse_expr("obj.method()");
        assert!(matches!(expr, Expression::Call(_)));
    }

    #[test]
    fn test_parse_delete() {
        let expr = parse_expr("delete obj.prop");
        assert!(matches!(expr, Expression::Unary(_)));
    }

    #[test]
    fn test_parse_void() {
        let expr = parse_expr("void 0");
        assert!(matches!(expr, Expression::Unary(_)));
    }
}
