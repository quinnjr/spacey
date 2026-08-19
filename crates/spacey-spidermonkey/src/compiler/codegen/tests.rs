//! Tests for the bytecode compiler.

use super::*;
use crate::parser::Parser;

fn compile_source(src: &str) -> Result<Bytecode, Error> {
    let mut parser = Parser::new(src);
    let program = parser.parse_program()?;
    let mut compiler = Compiler::new();
    compiler.compile(&program)
}

fn compile_ok(src: &str) -> Bytecode {
    compile_source(src).expect("Compilation should succeed")
}

#[test]
fn test_compiler_new() {
    let compiler = Compiler::new();
    assert!(compiler.bytecode.instructions.is_empty());
}

#[test]
fn test_compiler_default() {
    let compiler = Compiler::default();
    assert!(compiler.bytecode.instructions.is_empty());
}

#[test]
fn test_compile_empty_program() {
    let bytecode = compile_ok("");
    // Should emit LoadUndefined and Halt
    assert!(!bytecode.instructions.is_empty());
}

#[test]
fn test_compile_number_literal() {
    let bytecode = compile_ok("42;");
    assert!(!bytecode.instructions.is_empty());
}

#[test]
fn test_compile_string_literal() {
    let bytecode = compile_ok("'hello';");
    assert!(!bytecode.instructions.is_empty());
}

#[test]
fn test_compile_boolean_literals() {
    compile_ok("true;");
    compile_ok("false;");
}

#[test]
fn test_compile_null_undefined() {
    compile_ok("null;");
}

#[test]
fn test_compile_binary_add() {
    let bytecode = compile_ok("1 + 2;");
    assert!(!bytecode.instructions.is_empty());
}

#[test]
fn test_compile_binary_sub() {
    compile_ok("5 - 3;");
}

#[test]
fn test_compile_binary_mul() {
    compile_ok("2 * 3;");
}

#[test]
fn test_compile_binary_div() {
    compile_ok("10 / 2;");
}

#[test]
fn test_compile_comparison_operators() {
    compile_ok("1 < 2;");
    compile_ok("1 > 2;");
    compile_ok("1 <= 2;");
    compile_ok("1 >= 2;");
    compile_ok("1 == 2;");
    compile_ok("1 != 2;");
    compile_ok("1 === 2;");
    compile_ok("1 !== 2;");
}

#[test]
fn test_compile_unary_minus() {
    compile_ok("-42;");
}

#[test]
fn test_compile_unary_not() {
    compile_ok("!true;");
}

#[test]
fn test_compile_variable_declaration() {
    compile_ok("let x = 1;");
    compile_ok("const y = 2;");
    compile_ok("var z = 3;");
}

#[test]
fn test_compile_variable_use() {
    compile_ok("let x = 1; x;");
}

#[test]
fn test_compile_variable_assignment() {
    compile_ok("let x = 1; x = 2;");
}

#[test]
fn test_compile_multiple_variables() {
    compile_ok("let a = 1; let b = 2; a + b;");
}

#[test]
fn test_compile_if_statement() {
    compile_ok("if (true) { 1; }");
}

#[test]
fn test_compile_if_else() {
    compile_ok("if (true) { 1; } else { 2; }");
}

#[test]
fn test_compile_while_loop() {
    compile_ok("while (false) { 1; }");
}

#[test]
fn test_compile_for_loop() {
    compile_ok("for (let i = 0; i < 10; i = i + 1) { }");
}

#[test]
fn test_compile_function_declaration() {
    compile_ok("function f() { return 1; }");
}

#[test]
fn test_compile_function_call() {
    compile_ok("function f() { return 1; } f();");
}

#[test]
fn test_compile_function_with_params() {
    compile_ok("function add(a, b) { return a + b; }");
}

#[test]
fn test_compile_return_statement() {
    compile_ok("function f() { return; }");
    compile_ok("function f() { return 42; }");
}

#[test]
fn test_compile_block_statement() {
    compile_ok("{ let x = 1; }");
}

#[test]
fn test_compile_nested_blocks() {
    compile_ok("{ let x = 1; { let y = 2; } }");
}

#[test]
fn test_compile_expression_statement() {
    compile_ok("1 + 2;");
    compile_ok("f();");
}

#[test]
fn test_compile_array_literal() {
    compile_ok("let arr = [1, 2, 3];");
}

#[test]
fn test_compile_object_literal() {
    compile_ok("let obj = { x: 1 };");
}

#[test]
fn test_compile_member_expression() {
    compile_ok("let obj = { x: 1 }; obj.x;");
}

#[test]
fn test_compile_complex_expression() {
    compile_ok("1 + 2 * 3;");
    compile_ok("(1 < 2) === true;");
}

#[test]
fn test_compile_switch_debug() {
    let bytecode = compile_ok("var x; switch (1) { case 1: x = 'yes'; break; default: x = 'no'; }");
    println!("Switch bytecode:");
    for (i, inst) in bytecode.instructions.iter().enumerate() {
        println!("{}: {:?}", i, inst);
    }
    println!("\nConstants:");
    for (i, c) in bytecode.constants.iter().enumerate() {
        println!("{}: {:?}", i, c);
    }
}

#[test]
fn test_compile_no_return_function_debug() {
    let bytecode = compile_ok("function foo() { var x = 1; }");
    println!("Function bytecode:");
    for (i, inst) in bytecode.instructions.iter().enumerate() {
        println!("{}: {:?}", i, inst);
    }
}

#[test]
fn test_compile_call_nested() {
    let bytecode = compile_ok("function foo() { var x = 1; } function bar(x) { } bar(foo());");
    println!("Main bytecode:");
    for (i, inst) in bytecode.instructions.iter().enumerate() {
        println!("{}: {:?}", i, inst);
    }
    println!("\nConstants:");
    for (i, c) in bytecode.constants.iter().enumerate() {
        println!("{}: {:?}", i, c);
    }
}

#[test]
fn test_compile_function_with_var() {
    let bytecode = compile_ok("function foo() { var x; }");
    println!("Main bytecode:");
    for (i, inst) in bytecode.instructions.iter().enumerate() {
        println!("{}: {:?}", i, inst);
    }
    // Get the function from constant 0
    if let Some(crate::runtime::value::Value::Function(callable)) = bytecode.constants.get(0) {
        if let crate::runtime::function::Callable::Function(func) = callable.as_ref() {
            println!("\nFunction foo() bytecode:");
            for (i, inst) in func.bytecode.instructions.iter().enumerate() {
                println!("{}: {:?}", i, inst);
            }
        }
    }
}

#[test]
fn test_compile_obj_assign() {
    let bytecode = compile_ok("var obj = { city: 'NYC' }; obj.city = 'LA';");
    println!("Object assign bytecode:");
    for (i, inst) in bytecode.instructions.iter().enumerate() {
        println!("{}: {:?}", i, inst);
    }
    println!("\nConstants:");
    for (i, c) in bytecode.constants.iter().enumerate() {
        println!("{}: {:?}", i, c);
    }
}

#[test]
fn test_compile_closure_inner_function() {
    let bytecode = compile_ok(
        "function makeCounter() { var count = 0; return function() { count = count + 1; return count; }; }",
    );
    // Get the makeCounter function
    if let Some(crate::runtime::value::Value::Function(callable)) = bytecode.constants.get(0) {
        if let crate::runtime::function::Callable::Function(make_counter) = callable.as_ref() {
            println!("makeCounter bytecode:");
            for (i, inst) in make_counter.bytecode.instructions.iter().enumerate() {
                println!("{}: {:?}", i, inst);
            }
            // Find the inner function in makeCounter's constants
            for (i, c) in make_counter.bytecode.constants.iter().enumerate() {
                if let crate::runtime::value::Value::Function(inner_callable) = c {
                    if let crate::runtime::function::Callable::Function(inner) =
                        inner_callable.as_ref()
                    {
                        println!("\nInner function bytecode:");
                        for (j, inst) in inner.bytecode.instructions.iter().enumerate() {
                            println!("{}: {:?}", j, inst);
                        }
                    }
                }
            }
        }
    }
}
