// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! CommonJS require() implementation

use crate::error::Result;
use crate::module_system::loader::ModuleLoader;
use spacey_spidermonkey::Value;
use std::path::Path;

/// The require() function for CommonJS modules
pub fn require(loader: &mut ModuleLoader, specifier: &str, parent_path: &Path) -> Result<Value> {
    loader.load(specifier, parent_path)
}

/// require.resolve() - get the resolved path without loading
pub fn require_resolve(
    _loader: &ModuleLoader,
    specifier: &str,
    parent_path: &Path,
) -> Result<String> {
    use crate::module_system::resolver::{ModuleResolver, ResolveResult};

    let resolver = ModuleResolver::new();
    let resolved = resolver.resolve(specifier, parent_path)?;

    match resolved {
        ResolveResult::BuiltIn(name) => Ok(format!("node:{}", name)),
        ResolveResult::BuiltInSubpath { module: _, subpath } => Ok(format!("node:{}", subpath)),
        ResolveResult::File(path) | ResolveResult::Json(path) | ResolveResult::Native(path) => {
            Ok(path.display().to_string())
        }
    }
}

/// Check if a module specifier is a built-in module
pub fn is_builtin(specifier: &str) -> bool {
    let resolver = crate::module_system::resolver::ModuleResolver::new();
    resolver.is_builtin(specifier)
}

/// Get all built-in module names
pub fn builtin_modules() -> Vec<&'static str> {
    crate::module_system::resolver::ModuleResolver::builtin_modules()
}

/// require.cache - the module cache object
pub fn require_cache(loader: &ModuleLoader) -> Value {
    let cache = loader.cache();
    let mut obj = std::collections::HashMap::new();

    for path in cache.keys() {
        if let Some(module) = cache.get(&path) {
            obj.insert(path.display().to_string(), module.exports.clone());
        }
    }

    Value::NativeObject(obj)
}

/// require.main - the main module
pub fn require_main() -> Value {
    // Would be set when the main module is loaded
    Value::Undefined
}
