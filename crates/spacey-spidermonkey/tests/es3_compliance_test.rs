//! ES3 Compliance Integration Tests
//!
//! This module runs the ES3 compliance test suite from tests/es3_compliance.js

use spacey_spidermonkey::Engine;
use std::fs;
use std::path::Path;

/// Run a JavaScript file and capture the output
fn run_js_file(path: &str) -> Result<String, String> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    let mut engine = Engine::new();
    engine.eval(&source).map_err(|e| format!("{:?}", e))?;

    // For now, just return success - in a full implementation,
    // we'd capture console output
    Ok("Execution completed".to_string())
}

#[test]
#[ignore = "requires full engine implementation"]
fn test_es3_compliance_full() {
    let test_file = "../../tests/es3_compliance.js";

    if !Path::new(test_file).exists() {
        panic!("ES3 compliance test file not found: {}", test_file);
    }

    match run_js_file(test_file) {
        Ok(_) => println!("ES3 compliance tests executed successfully"),
        Err(e) => panic!("ES3 compliance tests failed: {}", e),
    }
}

// Individual feature tests that can run with current engine state

#[test]
fn test_es3_arithmetic() {
    let mut engine = Engine::new();

    // Basic arithmetic
    assert_eq!(engine.eval("5 + 3;").unwrap().to_string(), "8");
    assert_eq!(engine.eval("10 - 4;").unwrap().to_string(), "6");
    assert_eq!(engine.eval("6 * 7;").unwrap().to_string(), "42");
    assert_eq!(engine.eval("15 / 3;").unwrap().to_string(), "5");
    assert_eq!(engine.eval("17 % 5;").unwrap().to_string(), "2");
}

#[test]
fn test_es3_comparison() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("5 == 5;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("5 != 3;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("5 === 5;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("5 < 10;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("10 > 5;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("5 <= 5;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("5 >= 5;").unwrap().to_string(), "true");
}

#[test]
fn test_es3_logical() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("true && true;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("true && false;").unwrap().to_string(), "false");
    assert_eq!(engine.eval("false || true;").unwrap().to_string(), "true");
    assert_eq!(engine.eval("false || false;").unwrap().to_string(), "false");
    assert_eq!(engine.eval("!true;").unwrap().to_string(), "false");
    assert_eq!(engine.eval("!false;").unwrap().to_string(), "true");
}

#[test]
fn test_es3_variables() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("var x = 42; x;").unwrap().to_string(), "42");
    assert_eq!(
        engine.eval("var a = 1, b = 2; a + b;").unwrap().to_string(),
        "3"
    );
}

#[test]
fn test_es3_strings() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("\"hello\".length;").unwrap().to_string(), "5");
    assert_eq!(
        engine
            .eval("\"hello\" + \" \" + \"world\";")
            .unwrap()
            .to_string(),
        "hello world"
    );
}

#[test]
fn test_es3_typeof() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("typeof 42;").unwrap().to_string(), "number");
    assert_eq!(
        engine.eval("typeof \"hello\";").unwrap().to_string(),
        "string"
    );
    assert_eq!(engine.eval("typeof true;").unwrap().to_string(), "boolean");
    assert_eq!(
        engine.eval("typeof undefined;").unwrap().to_string(),
        "undefined"
    );
}

#[test]
fn test_es3_if_statement() {
    let mut engine = Engine::new();

    assert_eq!(
        engine
            .eval("var x; if (true) { x = 1; } else { x = 2; } x;")
            .unwrap()
            .to_string(),
        "1"
    );
    assert_eq!(
        engine
            .eval("var y; if (false) { y = 1; } else { y = 2; } y;")
            .unwrap()
            .to_string(),
        "2"
    );
}

#[test]
fn test_es3_for_loop() {
    let mut engine = Engine::new();

    assert_eq!(
        engine
            .eval("var sum = 0; for (var i = 1; i <= 5; i++) { sum = sum + i; } sum;")
            .unwrap()
            .to_string(),
        "15"
    );
}

#[test]
fn test_es3_while_loop() {
    let mut engine = Engine::new();

    assert_eq!(
        engine
            .eval("var sum = 0; var i = 1; while (i <= 5) { sum = sum + i; i = i + 1; } sum;")
            .unwrap()
            .to_string(),
        "15"
    );
}

#[test]
fn test_es3_array() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("[1, 2, 3].length;").unwrap().to_string(), "3");
    assert_eq!(engine.eval("[1, 2, 3][0];").unwrap().to_string(), "1");
    assert_eq!(engine.eval("[1, 2, 3][2];").unwrap().to_string(), "3");
}

#[test]
fn test_es3_object() {
    let mut engine = Engine::new();

    assert_eq!(
        engine
            .eval("var obj = { x: 42 }; obj.x;")
            .unwrap()
            .to_string(),
        "42"
    );
    assert_eq!(
        engine
            .eval("var obj = { a: 1, b: 2 }; obj.a + obj.b;")
            .unwrap()
            .to_string(),
        "3"
    );
}

#[test]
fn test_es3_function() {
    let mut engine = Engine::new();

    assert_eq!(
        engine
            .eval("function add(a, b) { return a + b; } add(2, 3);")
            .unwrap()
            .to_string(),
        "5"
    );
}

#[test]
fn test_es3_math() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("Math.abs(-5);").unwrap().to_string(), "5");
    assert_eq!(engine.eval("Math.floor(3.7);").unwrap().to_string(), "3");
    assert_eq!(engine.eval("Math.ceil(3.2);").unwrap().to_string(), "4");
    assert_eq!(engine.eval("Math.pow(2, 3);").unwrap().to_string(), "8");
}

#[test]
fn test_es3_parseint() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("parseInt(\"42\");").unwrap().to_string(), "42");
    assert_eq!(
        engine.eval("parseInt(\"ff\", 16);").unwrap().to_string(),
        "255"
    );
}

#[test]
fn test_es3_parsefloat() {
    let mut engine = Engine::new();

    assert_eq!(
        engine.eval("parseFloat(\"3.14\");").unwrap().to_string(),
        "3.14"
    );
}

#[test]
fn test_es3_isnan() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("isNaN(NaN);").unwrap().to_string(), "true");
    assert_eq!(engine.eval("isNaN(42);").unwrap().to_string(), "false");
}

#[test]
fn test_es3_isfinite() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("isFinite(42);").unwrap().to_string(), "true");
    assert_eq!(
        engine.eval("isFinite(Infinity);").unwrap().to_string(),
        "false"
    );
}

#[test]
fn test_es3_ternary() {
    let mut engine = Engine::new();

    assert_eq!(
        engine.eval("true ? \"yes\" : \"no\";").unwrap().to_string(),
        "yes"
    );
    assert_eq!(
        engine
            .eval("false ? \"yes\" : \"no\";")
            .unwrap()
            .to_string(),
        "no"
    );
}

#[test]
fn test_es3_increment_decrement() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("var x = 5; x++; x;").unwrap().to_string(), "6");
    assert_eq!(engine.eval("var y = 5; y--; y;").unwrap().to_string(), "4");
}

#[test]
fn test_es3_bitwise() {
    let mut engine = Engine::new();

    assert_eq!(engine.eval("5 & 3;").unwrap().to_string(), "1");
    assert_eq!(engine.eval("5 | 3;").unwrap().to_string(), "7");
    assert_eq!(engine.eval("5 ^ 3;").unwrap().to_string(), "6");
    assert_eq!(engine.eval("4 << 1;").unwrap().to_string(), "8");
    assert_eq!(engine.eval("8 >> 1;").unwrap().to_string(), "4");
}

#[test]
fn test_es3_multi_param_function() {
    let mut engine = Engine::new();

    // Test a function with multiple parameters and compound assignment
    let code = r#"
var testsPassed = 0;

function assert(condition, message) {
    if (condition) {
        testsPassed = testsPassed + 1;
    }
}

assert(true, "test");
testsPassed;
"#;

    let result = engine.eval(code);
    println!("Result: {:?}", result);
    assert_eq!(result.unwrap().to_string(), "1");
}

#[test]
fn test_es3_function_arg_passing() {
    let mut engine = Engine::new();

    // Test basic function parameter passing
    let code = r#"
function checkArg(x) {
    return x;
}
checkArg(true);
"#;

    let result = engine.eval(code).unwrap();
    println!("checkArg(true) = {:?}", result);
    assert_eq!(result.to_string(), "true");
}

#[test]
fn test_es3_global_var_update_in_function() {
    let mut engine = Engine::new();

    // Test updating global variables from within a function
    let code = r#"
var counter = 0;

function increment() {
    counter = counter + 1;
}

increment();
counter;
"#;

    let result = engine.eval(code).unwrap();
    println!("counter after increment() = {:?}", result);
    assert_eq!(result.to_string(), "1");
}

#[test]
fn test_debug_global_access() {
    let mut engine = Engine::new();

    // Simple global variable read after function call
    let code = r#"
var x = 10;

function read_x() {
    return x;
}

read_x();
"#;

    let result = engine.eval(code).unwrap();
    println!("read_x() = {:?}", result);
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_parse_console_log() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#"console.log("test");"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_string_method() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#""=".repeat(60);"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_assertEqual() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#"
function assertEqual(actual, expected, message) {
    if (actual === expected) {
        testsPassed = testsPassed + 1;
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_for_with_increment() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#"for (var i = 1; i <= 5; i++) { }"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_first_50_lines() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();
    let lines: Vec<&str> = source.lines().take(50).collect();
    let partial = lines.join("\n");

    let mut parser = Parser::new(&partial);
    let result = parser.parse_program();
    println!("Parse result of first 50 lines: {:?}", result.is_ok());
    if let Err(e) = &result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_parse_progressive() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();

    // Try to parse the whole file
    let mut parser = Parser::new(&source);
    match parser.parse_program() {
        Ok(_) => println!("Full file parses successfully!"),
        Err(e) => {
            // Find approximately where the error is
            let lines: Vec<&str> = source.lines().collect();
            for end in (100..lines.len()).step_by(50) {
                let partial = lines[..end].join("\n");
                let mut p = Parser::new(&partial);
                if p.parse_program().is_err() {
                    println!(
                        "Parse error somewhere in lines {}-{}: {:?}",
                        end - 50,
                        end,
                        e
                    );
                    println!(
                        "Context:\n{}",
                        lines[end.saturating_sub(5)..end.min(lines.len())].join("\n")
                    );
                    break;
                }
            }
            panic!("Parse failed: {:?}", e);
        }
    }
}

#[test]
fn test_parse_comma_operator() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#"var commaResult = (1, 2, 3);"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_in_operator() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#""x" in obj;"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_parse_up_to_line_600() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();
    let lines: Vec<&str> = source.lines().take(600).collect();
    let partial = lines.join("\n");

    let mut parser = Parser::new(&partial);
    let result = parser.parse_program();
    if let Err(e) = &result {
        println!("Parse error: {:?}", e);
    } else {
        println!("Lines 1-600 parse OK");
    }
    assert!(result.is_ok());
}

#[test]
fn test_parse_around_regex() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();
    let lines: Vec<&str> = source.lines().collect();

    // Try lines around the regex area
    for end in [650, 680, 685, 686, 687] {
        if end > lines.len() {
            break;
        }
        let partial = lines[..end].join("\n");
        let mut parser = Parser::new(&partial);
        if let Err(e) = parser.parse_program() {
            println!("Parse error at line {}: {:?}", end, e);
            println!("Line {}: {}", end, lines[end - 1]);
        } else {
            println!("Lines 1-{}: OK", end);
        }
    }
}

#[test]
fn test_full_parse() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();
    let lines: Vec<&str> = source.lines().collect();

    // Binary search for the error line
    for end in (600..=lines.len()).step_by(10) {
        let partial = lines[..end.min(lines.len())].join("\n");
        let mut parser = Parser::new(&partial);
        if let Err(e) = parser.parse_program() {
            println!("Parse error around line {}: {:?}", end, e);
            let start = (end as i32 - 5).max(0) as usize;
            println!(
                "Context:\n{}",
                lines[start..end.min(lines.len())].join("\n")
            );
            return;
        }
    }
    println!("Full file ({} lines) parses OK!", lines.len());
}

#[test]
fn test_parse_string_concat() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#"console.log("\n" + "=".repeat(60));"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_find_exact_error_line() {
    use spacey_spidermonkey::parser::Parser;
    use std::fs;

    let source = fs::read_to_string("../../tests/es3_compliance.js").unwrap();
    let lines: Vec<&str> = source.lines().collect();

    for end in 710..=lines.len() {
        let partial = lines[..end].join("\n");
        let mut parser = Parser::new(&partial);
        if let Err(e) = parser.parse_program() {
            println!("First parse error at line {}: {:?}", end, e);
            println!("Line {}: {}", end, lines[end - 1]);
            return;
        }
    }
    println!("Full file parses OK");
}

#[test]
fn test_parse_paren_addition() {
    use spacey_spidermonkey::parser::Parser;

    let code = r#""str" + (a + b);"#;
    let mut parser = Parser::new(code);
    let result = parser.parse_program();
    println!("Parse result: {:?}", result);
    assert!(result.is_ok());
}
