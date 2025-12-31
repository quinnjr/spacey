//! Simple example demonstrating basic Spacey-Servo integration.
//!
//! This example shows basic JavaScript execution with the Spacey engine
//! in a Servo-compatible context.

use spacey_servo::SpaceyServo;

fn main() {
    println!("🚀 Spacey-Servo Simple Example\n");

    // Create a Spacey-Servo instance
    let servo = SpaceyServo::new();
    println!("✓ Created Spacey-Servo instance\n");

    // Test basic arithmetic
    println!("--- Basic Arithmetic ---");
    match servo.eval("2 + 2;") {
        Ok(result) => println!("2 + 2 = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval("10 * 5;") {
        Ok(result) => println!("10 * 5 = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval("100 / 4;") {
        Ok(result) => println!("100 / 4 = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test variables
    println!("\n--- Variables ---");
    match servo.eval("var x = 42;") {
        Ok(_) => println!("✓ Created variable x = 42"),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval("x;") {
        Ok(result) => println!("x = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test multiple operations
    println!("\n--- Multiple Operations ---");
    servo.eval("var a = 10;").ok();
    servo.eval("var b = 20;").ok();
    
    match servo.eval("a + b;") {
        Ok(result) => println!("a + b = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test boolean operations
    println!("\n--- Boolean Operations ---");
    match servo.eval("true;") {
        Ok(result) => println!("true = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval("false;") {
        Ok(result) => println!("false = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval("1 < 2;") {
        Ok(result) => println!("1 < 2 = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test string operations
    println!("\n--- String Operations ---");
    match servo.eval(r#""Hello";"#) {
        Ok(result) => println!("String: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match servo.eval(r#"var greeting = "World";"#) {
        Ok(_) => println!("✓ Created string variable"),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n✨ Example completed!");
}
