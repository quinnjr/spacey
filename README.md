# Spacey

**An experimental JavaScript engine written in Rust, targeting adoption by the [Servo](https://servo.org/) web browser project.**

> ⚠️ **Experimental Software**: Spacey is in early development and is not yet suitable for production use. APIs and internals are subject to change.

## Overview

Spacey is a from-scratch JavaScript engine implementation in Rust, inspired by Mozilla's SpiderMonkey. The project aims to provide a modern, memory-safe JavaScript runtime that can serve as Servo's JavaScript engine, replacing the current SpiderMonkey integration with a pure-Rust solution.

### Why Spacey?

- **Pure Rust**: No C++ dependencies, enabling tighter integration with Servo's Rust codebase
- **Memory Safety**: Leverages Rust's ownership model for a safer JavaScript runtime
- **Modern Architecture**: Clean-slate design informed by decades of JS engine research
- **Servo-First**: Designed with Servo's requirements and architecture in mind

## Project Goals

1. **ECMAScript Compliance**: Full ES2024+ specification compliance
2. **Performance**: Competitive performance through bytecode compilation and JIT
3. **Embeddability**: Clean, safe Rust API for embedding in Servo and other applications
4. **Web Platform Integration**: Seamless integration with Servo's DOM and Web APIs

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        spacey                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              spacey-spidermonkey                     │   │
│  │  ┌─────────┐  ┌────────┐  ┌──────────┐  ┌────────┐ │   │
│  │  │  Lexer  │→ │ Parser │→ │ Compiler │→ │   VM   │ │   │
│  │  └─────────┘  └────────┘  └──────────┘  └────────┘ │   │
│  │       ↓            ↓            ↓            ↓      │   │
│  │   Tokens         AST       Bytecode      Execution │   │
│  │                                                     │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │  Runtime: Values, Objects, GC, Built-ins     │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              spacey-node (optional)                  │   │
│  │         Node.js bindings via napi-rs                │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Workspace Structure

| Crate | Description |
|-------|-------------|
| `spacey` | CLI and REPL interface |
| `spacey-spidermonkey` | Core JavaScript engine library |
| `spacey-node` | Node.js native addon bindings |

## Current Status

| Component | Status |
|-----------|--------|
| Lexer | 🚧 In Progress |
| Parser | 🚧 In Progress |
| AST | 🚧 In Progress |
| Bytecode Compiler | 🚧 In Progress |
| VM Interpreter | 🚧 In Progress |
| Built-in Objects | 📋 Planned |
| Garbage Collector | 📋 Planned |
| JIT Compiler | 📋 Planned |
| Servo Integration | 📋 Planned |

See [TODO.md](./TODO.md) for the detailed implementation roadmap.

## Building

### Prerequisites

- Rust 2024 edition (nightly)
- Cargo

### Build

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run the REPL
cargo run
```

### Node.js Bindings (Optional)

```bash
cd crates/spacey-node
pnpm install
pnpm build
```

## Usage

### As a Library (Rust)

```rust
use spacey_spidermonkey::Engine;

fn main() {
    let mut engine = Engine::new();

    match engine.eval("1 + 2 * 3") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### From Node.js

```javascript
const spacey = require('spacey-node');

const engine = new spacey.Engine();
const result = engine.eval('Math.sqrt(16)');
console.log(result); // { valueType: 'number', value: '4' }
```

## Servo Integration

Spacey is being developed with Servo integration as a primary goal. The planned integration points include:

- **Script Thread**: Direct integration with Servo's script execution pipeline
- **DOM Bindings**: Rust-native bindings to Servo's DOM implementation
- **Web APIs**: Support for Web platform APIs through Servo's infrastructure
- **Memory Management**: Coordination with Servo's memory management strategies

### Comparison with Current SpiderMonkey Integration

| Aspect | SpiderMonkey (current) | Spacey (planned) |
|--------|----------------------|------------------|
| Language | C++ with Rust bindings | Pure Rust |
| Integration | FFI boundary | Native Rust API |
| Memory | Separate GC | Integrated with Servo |
| Build | Complex C++ toolchain | Cargo-only |

## Contributing

Spacey welcomes contributions! Areas where help is especially needed:

- ECMAScript specification compliance
- Performance optimization
- Test262 test suite integration
- Documentation

Please see the [TODO.md](./TODO.md) for specific tasks and milestones.

## Related Projects

- [Servo](https://servo.org/) - The browser engine Spacey targets
- [SpiderMonkey](https://spidermonkey.dev/) - Mozilla's JS engine (architectural inspiration)
- [Boa](https://github.com/boa-dev/boa) - Another Rust JS engine
- [Deno](https://deno.land/) - Rust-based JS/TS runtime using V8

## License

This project is licensed under the [Mozilla Public License 2.0](./LICENSE).

Copyright © 2025 Joseph R. Quinn

---

<p align="center">
  <strong>Spacey</strong> — A JavaScript engine for the Rust-powered web
</p>

