// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Module loader - reads and compiles modules

use crate::error::{NodeError, Result};
use crate::module_system::cache::{CachedModule, ModuleCache};
use crate::module_system::resolver::{ModuleResolver, ResolveResult};
use spacey_spidermonkey::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Module loader
pub struct ModuleLoader {
    /// Module resolver
    resolver: ModuleResolver,
    /// Module cache
    cache: ModuleCache,
    /// Stack of currently loading modules (for circular dependency detection)
    loading_stack: Vec<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        Self {
            resolver: ModuleResolver::new(),
            cache: ModuleCache::new(),
            loading_stack: Vec::new(),
        }
    }

    /// Load a module
    pub fn load(&mut self, specifier: &str, parent_path: &Path) -> Result<Value> {
        // Resolve the module
        let resolved = self.resolver.resolve(specifier, parent_path)?;

        match resolved {
            ResolveResult::BuiltIn(name) => {
                // Return built-in module exports
                self.load_builtin(&name, None)
            }
            ResolveResult::BuiltInSubpath { module, subpath } => {
                // Return built-in module with subpath (e.g., fs/promises)
                self.load_builtin(&module, Some(&subpath))
            }
            ResolveResult::File(path) => {
                // Load JavaScript file
                self.load_js_file(&path, Some(parent_path))
            }
            ResolveResult::Json(path) => {
                // Load JSON file
                self.load_json_file(&path)
            }
            ResolveResult::Native(path) => {
                // Native addons not supported
                Err(NodeError::ModuleResolution {
                    module: path.display().to_string(),
                    reason: "Native addons (.node) are not supported".to_string(),
                })
            }
        }
    }

    /// Load a built-in module
    fn load_builtin(&self, name: &str, subpath: Option<&str>) -> Result<Value> {
        // Built-in modules are registered separately
        // Return a marker that the runtime will resolve
        if let Some(subpath) = subpath {
            // Subpath import (e.g., node:fs/promises)
            Ok(Value::String(format!("__builtin__:{}:{}", name, subpath)))
        } else {
            Ok(Value::String(format!("__builtin__:{}", name)))
        }
    }

    /// Load a JavaScript file
    fn load_js_file(&mut self, path: &Path, parent: Option<&Path>) -> Result<Value> {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Check cache
        if let Some(cached) = self.cache.get(&abs_path) {
            return Ok(cached.exports);
        }

        // Check for circular dependency
        if self.loading_stack.contains(&abs_path) {
            // Return partial exports (allows circular requires like Node.js)
            return Ok(Value::Object(0)); // Placeholder empty exports
        }

        // Push to loading stack
        self.loading_stack.push(abs_path.clone());

        // Read file
        let source = std::fs::read_to_string(&abs_path)?;

        // Create module wrapper
        let dirname = abs_path.parent().unwrap_or(Path::new("."));
        let wrapped = self.wrap_module(&source, &abs_path, dirname);

        // Create initial cache entry with empty exports
        let module_entry = CachedModule {
            exports: Value::Object(0), // Empty object placeholder
            filename: abs_path.clone(),
            loaded: false,
            children: Vec::new(),
            parent: parent.map(|p| p.to_path_buf()),
        };
        self.cache.set(abs_path.clone(), module_entry);

        // The wrapped code would be executed by the engine
        // For now, return a placeholder
        let exports = Value::Object(0);

        // Update cache with final exports
        let final_entry = CachedModule {
            exports: exports.clone(),
            filename: abs_path.clone(),
            loaded: true,
            children: Vec::new(),
            parent: parent.map(|p| p.to_path_buf()),
        };
        self.cache.set(abs_path.clone(), final_entry);

        // Pop from loading stack
        self.loading_stack.pop();

        Ok(exports)
    }

    /// Load a JSON file
    fn load_json_file(&mut self, path: &Path) -> Result<Value> {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Check cache
        if let Some(cached) = self.cache.get(&abs_path) {
            return Ok(cached.exports);
        }

        // Read and parse JSON
        let content = std::fs::read_to_string(&abs_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        // Convert to spacey Value
        let value = json_to_value(&json);

        // Cache
        let module_entry = CachedModule {
            exports: value.clone(),
            filename: abs_path.clone(),
            loaded: true,
            children: Vec::new(),
            parent: None,
        };
        self.cache.set(abs_path, module_entry);

        Ok(value)
    }

    /// Wrap source code in CommonJS module wrapper
    fn wrap_module(&self, source: &str, filename: &Path, dirname: &Path) -> String {
        format!(
            r#"(function(exports, require, module, __filename, __dirname) {{
{}
}});"#,
            source
        )
    }

    /// Get the module cache
    pub fn cache(&self) -> &ModuleCache {
        &self.cache
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Check if a module is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.resolver.is_builtin(name)
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert serde_json::Value to spacey Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let mut obj: std::collections::HashMap<String, Value> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), json_to_value(v)))
                .collect();
            obj.insert("length".to_string(), Value::Number(arr.len() as f64));
            Value::NativeObject(obj)
        }
        serde_json::Value::Object(obj) => {
            let map: std::collections::HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::NativeObject(map)
        }
    }
}
