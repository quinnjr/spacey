//! Statement compilation documentation.
//!
//! This module documents the statement compilation logic in `mod.rs`.
//! Statements are compiled to bytecode sequences that modify VM state.
//!
//! ## Statement Compilation Overview
//!
//! | Statement | Key Operations | Notes |
//! |-----------|----------------|-------|
//! | `var/let/const` | `DefineLocal`, `SetGlobal` | Hoisted for `var` |
//! | `if/else` | `JumpIfFalse`, `Jump` | Conditional branching |
//! | `while` | `JumpIfFalse`, `Jump` (back) | Loop with condition |
//! | `do-while` | `JumpIfTrue` | Loop with post-condition |
//! | `for` | Multiple jumps | Init, test, update, body |
//! | `for-in` | `ForInInit`, `ForInNext` | Object property iteration |
//! | `switch` | Multiple `JumpIfFalse` | Case matching |
//! | `try/catch/finally` | `TryStart`, `TryEnd`, `Catch`, `Finally` | Exception handling |
//! | `return` | `Return` | Function exit |
//! | `break/continue` | `Jump` (patched later) | Loop control |
//! | `throw` | `Throw` | Exception throwing |
//!
//! ## Variable Hoisting
//!
//! ES3 `var` declarations are hoisted to the top of their containing function:
//!
//! ```text
//! // Input:
//! console.log(x);  // undefined, not error
//! var x = 5;
//!
//! // Effectively:
//! var x;           // hoisted declaration
//! console.log(x);  // undefined
//! x = 5;           // assignment stays in place
//! ```
//!
//! The compiler performs two passes:
//! 1. `collect_hoisted_var_names` - Find all `var` declarations
//! 2. `hoist_declarations` - Pre-declare variables with `undefined`
//!
//! ## Control Flow Compilation
//!
//! ### If Statement
//!
//! ```text
//! if (condition) { then } else { else }
//!
//! Bytecode:
//!   [condition bytecode]
//!   JumpIfFalse -> else_label
//!   [then bytecode]
//!   Jump -> end_label
//! else_label:
//!   [else bytecode]
//! end_label:
//! ```
//!
//! ### While Loop
//!
//! ```text
//! while (condition) { body }
//!
//! Bytecode:
//! start_label:
//!   [condition bytecode]
//!   JumpIfFalse -> end_label
//!   [body bytecode]
//!   Jump -> start_label
//! end_label:
//! ```
//!
//! ### For-In Loop
//!
//! ```text
//! for (let key in obj) { body }
//!
//! Bytecode:
//!   [obj bytecode]
//!   ForInInit
//! loop_label:
//!   ForInNext -> end_label (jumps when exhausted)
//!   DefineLocal key
//!   [body bytecode]
//!   Jump -> loop_label
//! end_label:
//! ```
//!
//! ### Try-Catch-Finally
//!
//! ```text
//! try { body } catch (e) { handler } finally { cleanup }
//!
//! Bytecode:
//!   TryStart catch_label, finally_label
//!   [body bytecode]
//!   TryEnd
//!   Jump -> finally_label (or end)
//! catch_label:
//!   Catch
//!   DefineLocal e
//!   [handler bytecode]
//!   Jump -> finally_label (or end)
//! finally_label:
//!   Finally
//!   [cleanup bytecode]
//! end_label:
//! ```
//!
//! ## Break/Continue Handling
//!
//! Break and continue statements require jump patching because the target
//! address isn't known when the break/continue is compiled:
//!
//! ```text
//! while (true) {
//!     if (done) break;  // Jump address unknown here
//!     work();
//! }
//! // break jumps here - address known after loop compilation
//! ```
//!
//! The compiler uses `LoopContext` to track pending jumps:
//! - `enter_loop()` - Push new context
//! - `emit_break()` / `emit_continue()` - Record jump location
//! - `exit_loop()` - Patch all recorded jumps

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
    fn test_compile_var() {
        let bc = compile("var x = 5;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_let() {
        let bc = compile("let x = 5;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_const() {
        let bc = compile("const x = 5;");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_if() {
        let bc = compile("if (true) { x = 1; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_if_else() {
        let bc = compile("if (x) { y = 1; } else { y = 2; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_while() {
        let bc = compile("while (x < 10) { x = x + 1; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_do_while() {
        let bc = compile("do { x = x + 1; } while (x < 10);");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_for() {
        let bc = compile("for (let i = 0; i < 10; i = i + 1) { }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_for_in() {
        let bc = compile("for (let k in obj) { }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_switch() {
        let bc = compile("switch (x) { case 1: break; default: break; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_try_catch() {
        let bc = compile("try { x = 1; } catch (e) { }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_try_finally() {
        let bc = compile("try { x = 1; } finally { cleanup(); }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_return() {
        let bc = compile("function f() { return 42; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_break() {
        let bc = compile("while (true) { break; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_continue() {
        let bc = compile("while (true) { continue; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_throw() {
        let bc = compile("throw new Error('oops');");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_block() {
        let bc = compile("{ let x = 1; let y = 2; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_labeled_break() {
        let bc = compile("outer: while (true) { break outer; }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_labeled_continue() {
        let bc = compile("outer: while (true) { while (true) { continue outer; } }");
        assert!(!bc.instructions.is_empty());
    }

    #[test]
    fn test_compile_with() {
        let bc = compile("with (obj) { x = 1; }");
        assert!(!bc.instructions.is_empty());
    }
}
