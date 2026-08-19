//! Statement parsing utilities and documentation.
//!
//! This module documents the statement parsing logic found in `parser.rs`.
//! The methods here parse JavaScript statements according to the ECMAScript spec.
//!
//! ## Statement Types
//!
//! | Statement | Method | ES Spec Section |
//! |-----------|--------|-----------------|
//! | Variable | `parse_variable_declaration` | 12.2 |
//! | Function | `parse_function_declaration` | 13 |
//! | If | `parse_if_statement` | 12.5 |
//! | Switch | `parse_switch_statement` | 12.11 |
//! | While | `parse_while_statement` | 12.6.2 |
//! | Do-While | `parse_do_while_statement` | 12.6.1 |
//! | For | `parse_for_statement` | 12.6.3 |
//! | For-In | `parse_for_statement` | 12.6.4 |
//! | Return | `parse_return_statement` | 12.9 |
//! | Break | `parse_break_statement` | 12.8 |
//! | Continue | `parse_continue_statement` | 12.7 |
//! | Throw | `parse_throw_statement` | 12.13 |
//! | Try | `parse_try_statement` | 12.14 |
//! | With | `parse_with_statement` | 12.10 |
//! | Debugger | inline in `parse_statement` | 12.15 |
//! | Block | `parse_block_statement` | 12.1 |
//! | Empty | inline in `parse_statement` | 12.3 |
//! | Expression | `parse_expression_statement` | 12.4 |
//!
//! ## Grammar Overview
//!
//! ```text
//! Statement :
//!     Block
//!     VariableStatement
//!     EmptyStatement
//!     ExpressionStatement
//!     IfStatement
//!     IterationStatement
//!     ContinueStatement
//!     BreakStatement
//!     ReturnStatement
//!     WithStatement
//!     LabelledStatement
//!     SwitchStatement
//!     ThrowStatement
//!     TryStatement
//!     DebuggerStatement
//! ```
//!
//! ## Example
//!
//! ```text
//! // Variable declaration
//! let x = 5;
//!
//! // If statement
//! if (x > 0) {
//!     console.log("positive");
//! } else {
//!     console.log("non-positive");
//! }
//!
//! // For loop
//! for (let i = 0; i < 10; i++) {
//!     console.log(i);
//! }
//!
//! // Try-catch
//! try {
//!     throw new Error("oops");
//! } catch (e) {
//!     console.log(e.message);
//! }
//! ```

// This module serves as documentation. The actual implementation is in parser.rs.
// Future refactoring could move statement parsing methods here.

#[cfg(test)]
mod tests {
    // Import from the main parser for testing
    use crate::ast::Statement;
    use crate::parser::Parser;

    fn parse_stmt(src: &str) -> Statement {
        let mut parser = Parser::new(src);
        parser.parse_statement().expect("Should parse")
    }

    #[test]
    fn test_parse_variable_let() {
        let stmt = parse_stmt("let x = 5;");
        assert!(matches!(stmt, Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_variable_const() {
        let stmt = parse_stmt("const PI = 3.14;");
        assert!(matches!(stmt, Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_variable_var() {
        let stmt = parse_stmt("var y = 'hello';");
        assert!(matches!(stmt, Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_if_simple() {
        let stmt = parse_stmt("if (true) { x = 1; }");
        assert!(matches!(stmt, Statement::If(_)));
    }

    #[test]
    fn test_parse_if_else() {
        let stmt = parse_stmt("if (x > 0) { y = 1; } else { y = 2; }");
        assert!(matches!(stmt, Statement::If(_)));
    }

    #[test]
    fn test_parse_while() {
        let stmt = parse_stmt("while (x < 10) { x = x + 1; }");
        assert!(matches!(stmt, Statement::While(_)));
    }

    #[test]
    fn test_parse_do_while() {
        let stmt = parse_stmt("do { x = x + 1; } while (x < 10);");
        assert!(matches!(stmt, Statement::DoWhile(_)));
    }

    #[test]
    fn test_parse_for() {
        let stmt = parse_stmt("for (let i = 0; i < 10; i = i + 1) { }");
        assert!(matches!(stmt, Statement::For(_)));
    }

    #[test]
    fn test_parse_for_in() {
        let stmt = parse_stmt("for (let k in obj) { }");
        assert!(matches!(stmt, Statement::ForIn(_)));
    }

    #[test]
    fn test_parse_break() {
        let stmt = parse_stmt("break;");
        assert!(matches!(stmt, Statement::Break));
    }

    #[test]
    fn test_parse_continue() {
        let stmt = parse_stmt("continue;");
        assert!(matches!(stmt, Statement::Continue));
    }

    #[test]
    fn test_parse_return() {
        let stmt = parse_stmt("return 42;");
        assert!(matches!(stmt, Statement::Return(_)));
    }

    #[test]
    fn test_parse_return_void() {
        let stmt = parse_stmt("return;");
        assert!(matches!(stmt, Statement::Return(_)));
    }

    #[test]
    fn test_parse_throw() {
        let stmt = parse_stmt("throw new Error('oops');");
        assert!(matches!(stmt, Statement::Throw(_)));
    }

    #[test]
    fn test_parse_try_catch() {
        let stmt = parse_stmt("try { x = 1; } catch (e) { }");
        assert!(matches!(stmt, Statement::Try(_)));
    }

    #[test]
    fn test_parse_try_finally() {
        let stmt = parse_stmt("try { x = 1; } finally { cleanup(); }");
        assert!(matches!(stmt, Statement::Try(_)));
    }

    #[test]
    fn test_parse_switch() {
        let stmt = parse_stmt("switch (x) { case 1: break; default: break; }");
        assert!(matches!(stmt, Statement::Switch(_)));
    }

    #[test]
    fn test_parse_block() {
        let stmt = parse_stmt("{ let x = 1; let y = 2; }");
        assert!(matches!(stmt, Statement::Block(_)));
    }

    #[test]
    fn test_parse_empty() {
        let stmt = parse_stmt(";");
        assert!(matches!(stmt, Statement::Empty));
    }

    #[test]
    fn test_parse_expression_statement() {
        let stmt = parse_stmt("console.log('hello');");
        assert!(matches!(stmt, Statement::Expression(_)));
    }

    #[test]
    fn test_parse_function_declaration() {
        let stmt = parse_stmt("function add(a, b) { return a + b; }");
        assert!(matches!(stmt, Statement::FunctionDeclaration(_)));
    }

    #[test]
    fn test_parse_debugger() {
        let stmt = parse_stmt("debugger;");
        assert!(matches!(stmt, Statement::Debugger));
    }

    #[test]
    fn test_parse_with() {
        let stmt = parse_stmt("with (obj) { x = 1; }");
        assert!(matches!(stmt, Statement::With(_)));
    }

    #[test]
    fn test_parse_labeled_break() {
        let stmt = parse_stmt("break outer;");
        assert!(matches!(stmt, Statement::BreakLabel(_)));
    }

    #[test]
    fn test_parse_labeled_continue() {
        let stmt = parse_stmt("continue loop;");
        assert!(matches!(stmt, Statement::ContinueLabel(_)));
    }
}
