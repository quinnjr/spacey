# spacey-servo

Servo browser integration with the Spacey JavaScript engine.

## Overview

This crate provides a bridge between the [Servo browser engine](https://servo.org/) and the Spacey JavaScript engine, replacing Servo's default SpiderMonkey integration with our custom implementation.

## Features

- **JavaScript Runtime**: Full ES3/ES5 JavaScript execution via Spacey
- **DOM Bindings**: JavaScript bindings for DOM objects (Window, Document, Element, etc.)
- **Event Loop**: Microtask and macrotask queue management
- **Servo Integration**: Drop-in replacement for Servo's JavaScript engine

## Architecture

```
┌─────────────────────────────────────┐
│         Servo Browser               │
│  ┌──────────────────────────────┐   │
│  │    HTML/CSS Rendering        │   │
│  └──────────────────────────────┘   │
│               ↕                      │
│  ┌──────────────────────────────┐   │
│  │    Spacey-Servo Bridge       │   │ ← This crate
│  │  • Runtime                   │   │
│  │  • DOM Bindings              │   │
│  │  • Event Loop                │   │
│  └──────────────────────────────┘   │
│               ↕                      │
│  ┌──────────────────────────────┐   │
│  │  Spacey JavaScript Engine    │   │
│  │  • Lexer                     │   │
│  │  • Parser                    │   │
│  │  • Compiler                  │   │
│  │  • VM                        │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

## Usage

### Basic Example

```rust
use spacey_servo::{SpaceyServo, DomBindings};

// Create a Spacey-Servo instance
let servo = SpaceyServo::new();

// Install DOM bindings
let bindings = DomBindings::new();
{
    let mut engine = servo.engine().write();
    bindings.install(&mut engine)?;
}

// Execute JavaScript with DOM access
servo.eval(r#"
    var doc = new Document();
    var element = doc.createElement('div');
    element.setAttribute('id', 'hello');
    console.log(element.getAttribute('id'));
"#)?;
```

### With Event Loop

```rust
use spacey_servo::SpaceyServo;

let servo = SpaceyServo::new();
let event_loop = servo.event_loop();

// Queue a microtask (like Promise.then)
event_loop.queue_microtask(|| {
    println!("Microtask executed!");
});

// Queue a macrotask (like setTimeout)
event_loop.queue_macrotask(|| {
    println!("Macrotask executed!");
});

// Run the event loop
event_loop.run();
```

## Components

### SpaceyRuntime

The main runtime that manages JavaScript execution and global scope initialization.

### DomBindings

Provides JavaScript bindings for DOM objects:
- `Window` - Browser window object
- `Document` - DOM document
- `Element` - DOM elements
- `Node` - DOM nodes
- `EventTarget` - Event handling

### EventLoop

Manages async operations with proper microtask/macrotask semantics:
- Microtasks (Promise callbacks, queueMicrotask)
- Macrotasks (setTimeout, setInterval, I/O)
- Proper execution order (microtasks drain before next macrotask)

## Servo Integration

To use Spacey with Servo, enable the `servo-integration` feature:

```toml
[dependencies]
spacey-servo = { version = "0.1", features = ["servo-integration"] }
```

This enables full Servo integration, including:
- Servo script traits implementation
- DOM bindings compatible with Servo's expectations
- Event loop integration with Servo's compositor

## Current Status

🚧 **Work in Progress**

This crate is under active development. Current capabilities:

- ✅ Basic JavaScript execution
- ✅ DOM object stubs (Window, Document, Element)
- ✅ Event loop with microtask/macrotask queues
- ✅ Basic event handling
- 🚧 Full Servo integration (in progress)
- 🚧 Complete DOM API coverage
- 🚧 Timer support (setTimeout/setInterval)
- 🚧 Fetch API
- 🚧 WebAssembly support

## Examples

Run the basic example:

```bash
cargo run --example basic
```

## Development

### Building

```bash
cargo build -p spacey-servo
```

### Testing

```bash
cargo test -p spacey-servo
```

### With Servo Integration

```bash
cargo build -p spacey-servo --features servo-integration
```

## License

MPL-2.0

## Contributing

See the main [Spacey repository](https://github.com/pegasusheavy/spacey) for contribution guidelines.
