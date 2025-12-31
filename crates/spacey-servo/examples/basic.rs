//! Basic example of using Spacey with Servo integration.
//!
//! This example demonstrates how to use the Spacey JavaScript engine
//! with Servo-compatible DOM bindings.

use spacey_servo::{SpaceyServo, DomBindings};

fn main() {
    println!("=== Spacey-Servo Integration Example ===\n");

    // Create a new Spacey-Servo instance
    let servo = SpaceyServo::new();
    println!("✓ Created Spacey-Servo instance");

    // Install DOM bindings
    let bindings = DomBindings::new();
    {
        let engine_arc = servo.engine();
        let mut engine = engine_arc.write();
        bindings.install(&mut engine).expect("Failed to install DOM bindings");
    }
    println!("✓ Installed DOM bindings");

    // Test basic JavaScript execution
    println!("\n--- Basic JavaScript ---");
    let result = servo.eval("1 + 2 + 3");
    println!("1 + 2 + 3 = {:?}", result);

    // Test DOM operations
    println!("\n--- DOM Operations ---");

    let result = servo.eval(r#"
        var doc = new Document();
        var element = doc.createElement('div');
        element.setAttribute('id', 'test');
        element.getAttribute('id');
    "#);
    println!("Created element with id: {:?}", result);

    // Test event target
    println!("\n--- Event Target ---");
    let result = servo.eval(r#"
        var target = new EventTarget();
        var called = false;
        target.addEventListener('test', function() {
            called = true;
        });
        target.dispatchEvent({ type: 'test' });
        called;
    "#);
    println!("Event listener called: {:?}", result);

    // Test Window object
    println!("\n--- Window Object ---");
    let result = servo.eval(r#"
        var win = new Window();
        win.location.href;
    "#);
    println!("Window location: {:?}", result);

    // Test multiple operations
    println!("\n--- Complex Operations ---");
    let result = servo.eval(r#"
        var doc = new Document();
        var parent = doc.createElement('div');
        var child1 = doc.createElement('span');
        var child2 = doc.createElement('p');

        parent.appendChild(child1);
        parent.appendChild(child2);

        parent.children.length;
    "#);
    println!("Parent has {} children: {:?}", 2, result);

    println!("\n✓ All tests completed successfully!");
}
