// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js module system implementation
//!
//! Implements both CommonJS `require()` and ECMAScript Modules (ESM).
//!
//! ## CommonJS
//! - `require()` function
//! - `module.exports` / `exports`
//! - Synchronous loading
//!
//! ## ESM
//! - `import` / `export` declarations
//! - `import.meta` object
//! - Dynamic `import()`
//! - `.mjs` / `.cjs` file extensions
//! - `package.json` "type" field

mod cache;
pub mod esm;
mod loader;
mod require;
mod resolver;

pub use cache::ModuleCache;
pub use esm::{EsmLoader, EsmModule, ImportMeta, ModuleType};
pub use loader::ModuleLoader;
pub use require::{
    builtin_modules, is_builtin, require, require_cache, require_main, require_resolve,
};
pub use resolver::{
    BUILTIN_MODULES, BUILTIN_SUBPATHS, BuiltinResolveResult, ModuleResolver, PROMISE_MODULES,
    ResolveResult,
};
