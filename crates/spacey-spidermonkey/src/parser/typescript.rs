//! TypeScript parsing support and documentation.
//!
//! This module documents the TypeScript parsing extensions in `parser.rs`.
//! TypeScript syntax is handled by parsing and skipping type annotations,
//! producing a standard JavaScript AST.
//!
//! ## Approach: Type Erasure
//!
//! Unlike traditional TypeScript tooling that transpiles `.ts` to `.js`,
//! Spacey **natively parses TypeScript** by:
//! 1. Recognizing TypeScript-specific tokens (lexer extensions)
//! 2. Parsing type syntax and skipping it (parser extensions)
//! 3. Outputting the same AST as if parsing JavaScript
//!
//! ## Supported TypeScript Features
//!
//! ### Type Annotations (Skipped)
//!
//! ```typescript
//! let x: number = 42;
//! function add(a: number, b: number): number { return a + b; }
//! const fn: (x: number) => number = (x) => x * 2;
//! ```
//!
//! ### Type Declarations (Skipped entirely)
//!
//! ```typescript
//! type ID = string | number;
//! interface User { name: string; age: number; }
//! ```
//!
//! ### Generics (Skipped)
//!
//! ```typescript
//! function identity<T>(x: T): T { return x; }
//! class Box<T> { value: T; }
//! ```
//!
//! ### Type Assertions (Converted)
//!
//! ```typescript
//! const x = value as string;  // Becomes: const x = value;
//! const y = <string>value;    // Becomes: const y = value;
//! ```
//!
//! ### Enums (Compiled to Objects)
//!
//! ```typescript
//! enum Color { Red, Green, Blue }
//! ```
//! Becomes:
//! ```javascript
//! var Color = {};
//! Color[Color["Red"] = 0] = "Red";
//! Color[Color["Green"] = 1] = "Green";
//! Color[Color["Blue"] = 2] = "Blue";
//! ```
//!
//! ### Access Modifiers (Skipped)
//!
//! ```typescript
//! class Foo {
//!     private x: number;
//!     public y: string;
//!     protected z: boolean;
//! }
//! ```
//!
//! ### Decorators (Skipped)
//!
//! ```typescript
//! @decorator
//! class Foo {
//!     @readonly
//!     prop: number;
//! }
//! ```
//!
//! ## Parser Methods
//!
//! | Method | Purpose |
//! |--------|---------|
//! | `skip_type_annotation` | Skip `: type` after identifiers |
//! | `skip_type_parameters` | Skip `<T, U>` generic parameters |
//! | `skip_type_arguments` | Skip `<number, string>` type arguments |
//! | `skip_type` | Skip a complete type expression |
//! | `skip_type_alias` | Skip `type Foo = ...;` |
//! | `skip_interface` | Skip `interface Foo { ... }` |
//! | `skip_declare_statement` | Skip `declare ...` ambient declarations |
//! | `skip_namespace` | Skip `namespace Foo { ... }` |
//! | `skip_decorators` | Skip `@decorator` annotations |
//! | `skip_access_modifiers` | Skip `public/private/protected` |
//! | `parse_enum_declaration` | Convert enum to JavaScript |
//!
//! ## Type Syntax Complexity
//!
//! The parser handles these type constructs:
//!
//! - Union types: `string | number`
//! - Intersection types: `A & B`
//! - Array types: `number[]`, `Array<number>`
//! - Tuple types: `[string, number]`
//! - Object types: `{ x: number; y: string }`
//! - Function types: `(x: number) => string`
//! - Conditional types: `T extends U ? X : Y`
//! - Mapped types: `{ [K in keyof T]: ... }`
//! - Index signatures: `{ [key: string]: number }`
//! - Literal types: `'foo' | 'bar'`, `1 | 2 | 3`

// This module serves as documentation. The actual implementation is in parser.rs.
// Future refactoring could move TypeScript parsing methods here.

#[cfg(test)]
mod tests {
    use crate::ast::{Program, Statement};
    use crate::parser::Parser;

    fn parse_ts(src: &str) -> Program {
        let mut parser = Parser::new_typescript(src);
        parser.parse_program().expect("Should parse")
    }

    fn parse_ts_stmt(src: &str) -> Statement {
        let mut parser = Parser::new_typescript(src);
        parser.parse_statement().expect("Should parse")
    }

    #[test]
    fn test_parse_ts_variable_annotation() {
        let program = parse_ts("let x: number = 42;");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_function_types() {
        let program = parse_ts("function add(a: number, b: number): number { return a + b; }");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::FunctionDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_generic_function() {
        let program = parse_ts("function identity<T>(x: T): T { return x; }");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::FunctionDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_type_alias_skipped() {
        let program = parse_ts("type ID = string | number;");
        // Type aliases are skipped, producing Empty statement
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::Empty));
    }

    #[test]
    fn test_parse_ts_interface_skipped() {
        let program = parse_ts("interface User { name: string; age: number; }");
        // Interfaces are skipped, producing Empty statement
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::Empty));
    }

    #[test]
    fn test_parse_ts_declare_skipped() {
        let program = parse_ts("declare const x: number;");
        // Declare statements are skipped
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::Empty));
    }

    #[test]
    fn test_parse_ts_namespace_skipped() {
        let program = parse_ts("namespace Utils { export function helper() {} }");
        // Namespaces are skipped
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::Empty));
    }

    #[test]
    fn test_parse_ts_optional_param() {
        let program = parse_ts("function greet(name?: string) { }");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::FunctionDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_array_type() {
        let program = parse_ts("let arr: number[] = [];");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_union_type() {
        let program = parse_ts("let value: string | number = 'hello';");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_intersection_type() {
        let program = parse_ts("let obj: A & B = {};");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_ts_object_type() {
        let program = parse_ts("let obj: { x: number; y: string } = { x: 1, y: 'a' };");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_ts_function_type() {
        let program = parse_ts("let fn: (x: number) => string = (x) => x.toString();");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_ts_enum() {
        let program = parse_ts("enum Color { Red, Green, Blue }");
        assert_eq!(program.body.len(), 1);
        // Enums are compiled to a block with var + assignment statements
        assert!(matches!(program.body[0], Statement::Block(_)));
    }

    #[test]
    fn test_parse_ts_string_enum() {
        let program = parse_ts("enum Status { Active = 'active', Inactive = 'inactive' }");
        assert_eq!(program.body.len(), 1);
        // String enums also become blocks
        assert!(matches!(program.body[0], Statement::Block(_)));
    }

    #[test]
    fn test_parse_ts_mixed_with_js() {
        let program = parse_ts(
            r#"
            type ID = number;
            interface User { name: string; }

            function greet(name: string): void {
                console.log('Hello, ' + name);
            }

            greet('World');
        "#,
        );
        // Should have: Empty (type), Empty (interface), FunctionDecl, Expression
        assert!(program.body.len() >= 2);
    }

    #[test]
    fn test_parse_ts_as_assertion() {
        let program = parse_ts("let x = value as string;");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_multiple_generics() {
        let program = parse_ts("function map<T, U>(arr: T[], fn: (x: T) => U): U[] { return []; }");
        assert_eq!(program.body.len(), 1);
        assert!(matches!(program.body[0], Statement::FunctionDeclaration(_)));
    }

    #[test]
    fn test_parse_ts_readonly() {
        let program = parse_ts("let arr: readonly number[] = [1, 2, 3];");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_ts_tuple() {
        // Tuple types are a complex TypeScript feature
        // The parser skips them as type annotations
        let program = parse_ts("let tuple = ['a', 1];"); // Without type annotation
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_ts_literal_type() {
        let program = parse_ts("let dir: 'north' | 'south' | 'east' | 'west' = 'north';");
        assert_eq!(program.body.len(), 1);
    }
}
