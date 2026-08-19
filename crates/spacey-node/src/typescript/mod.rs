//! TypeScript support for spacey-node.
//!
//! This module provides native TypeScript execution. Unlike traditional TypeScript
//! tooling that transpiles `.ts` to `.js` first, Spacey **natively parses and executes**
//! TypeScript by extending the lexer and parser to understand TypeScript syntax and
//! strip types at parse time.
//!
//! ## How It Works
//!
//! ```text
//! Traditional:  .ts → Transpiler → .js string → Parse JS → AST → Execute
//! Spacey:       .ts → Parse TS (strip types at parse time) → AST → Execute
//! ```
//!
//! ## Supported TypeScript Features
//!
//! - Type annotations (`let x: number = 42;`)
//! - Interface declarations (`interface Foo { ... }`)
//! - Type aliases (`type MyType = string | number;`)
//! - Generic type parameters (`function id<T>(x: T): T { ... }`)
//! - `as` type assertions (`x as string`)
//! - Non-null assertions (`x!`)
//! - Optional parameters (`function foo(x?: string) { ... }`)
//! - `declare` statements (`declare var x: number;`)
//! - Namespaces (`namespace N { ... }`)
//! - Enums (compiled to JavaScript objects)
//! - Access modifiers (`private`, `public`, `protected`, `readonly`)
//! - Abstract classes and methods
//! - Decorators (skipped at parse time)
//!
//! ## Usage
//!
//! TypeScript files are automatically detected by their extension and parsed
//! with TypeScript mode enabled:
//!
//! ```rust,ignore
//! use spacey_node::NodeRuntime;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut runtime = NodeRuntime::new(vec![]);
//!
//!     // TypeScript files are automatically handled
//!     runtime.run_file(Path::new("server.ts")).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Direct Engine Usage
//!
//! You can also use the engine's TypeScript methods directly:
//!
//! ```rust,ignore
//! use spacey_spidermonkey::Engine;
//!
//! let mut engine = Engine::new();
//!
//! // Evaluate TypeScript directly
//! let result = engine.eval_typescript("const x: number = 42; x;")?;
//!
//! // Or use auto-detection with files
//! let result = engine.eval_file_auto(Path::new("script.ts"))?;
//! ```

use std::path::Path;

/// Check if a file extension indicates a TypeScript file.
///
/// Returns `true` for `.ts`, `.tsx`, `.mts`, and `.cts` files.
pub fn is_typescript_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts" | "tsx" | "mts" | "cts") => true,
        _ => false,
    }
}

/// Check if a file extension indicates a JSX/TSX file.
///
/// Returns `true` for `.jsx` and `.tsx` files.
pub fn is_jsx_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jsx" | "tsx") => true,
        _ => false,
    }
}

/// TypeScript file extensions in resolution order.
pub const TS_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".mts", ".cts"];

/// All supported extensions including JavaScript.
pub const ALL_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_typescript_file() {
        assert!(is_typescript_file(Path::new("file.ts")));
        assert!(is_typescript_file(Path::new("file.tsx")));
        assert!(is_typescript_file(Path::new("file.mts")));
        assert!(is_typescript_file(Path::new("file.cts")));
        assert!(!is_typescript_file(Path::new("file.js")));
        assert!(!is_typescript_file(Path::new("file.jsx")));
        assert!(!is_typescript_file(Path::new("file.json")));
    }

    #[test]
    fn test_is_jsx_file() {
        assert!(is_jsx_file(Path::new("file.jsx")));
        assert!(is_jsx_file(Path::new("file.tsx")));
        assert!(!is_jsx_file(Path::new("file.js")));
        assert!(!is_jsx_file(Path::new("file.ts")));
    }

    #[test]
    fn test_extensions() {
        assert_eq!(TS_EXTENSIONS.len(), 4);
        assert_eq!(ALL_EXTENSIONS.len(), 8);
    }
}
