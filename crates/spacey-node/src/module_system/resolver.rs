// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Module path resolution (Node.js algorithm)

use crate::error::{NodeError, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Result of module resolution
#[derive(Debug, Clone)]
pub enum ResolveResult {
    /// Built-in module (fs, path, http, etc.)
    BuiltIn(String),
    /// Built-in module with subpath (fs/promises, stream/web, etc.)
    BuiltInSubpath { module: String, subpath: String },
    /// File module (resolved path)
    File(PathBuf),
    /// JSON file
    Json(PathBuf),
    /// Native addon (.node file)
    Native(PathBuf),
}

/// Result of resolving a built-in module
#[derive(Debug, Clone)]
pub struct BuiltinResolveResult {
    /// The base module name
    pub module: String,
    /// Optional subpath (e.g., "fs/promises" -> subpath = Some("fs/promises"))
    pub subpath: Option<String>,
}

/// All Node.js built-in modules (including experimental)
pub const BUILTIN_MODULES: &[&str] = &[
    // Core modules
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
    // Experimental modules
    "test",
    "sqlite",
    "sea",
];

/// Built-in modules that have promise variants (node:fs/promises)
pub const PROMISE_MODULES: &[&str] = &["fs", "dns", "readline", "stream", "timers"];

/// Built-in module subpaths (e.g., node:fs/promises)
pub const BUILTIN_SUBPATHS: &[(&str, &str)] = &[
    ("fs/promises", "fs"),
    ("dns/promises", "dns"),
    ("readline/promises", "readline"),
    ("stream/promises", "stream"),
    ("stream/consumers", "stream"),
    ("stream/web", "stream"),
    ("timers/promises", "timers"),
    ("util/types", "util"),
    ("path/posix", "path"),
    ("path/win32", "path"),
    ("assert/strict", "assert"),
];

/// Module resolver implementing Node.js resolution algorithm
pub struct ModuleResolver {
    /// Built-in module names
    builtins: Vec<String>,
    /// Built-in subpath mappings
    builtin_subpaths: std::collections::HashMap<String, String>,
    /// File extensions to try
    extensions: Vec<String>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        let builtins = BUILTIN_MODULES.iter().map(|s| s.to_string()).collect();

        let builtin_subpaths = BUILTIN_SUBPATHS
            .iter()
            .map(|(subpath, parent)| (subpath.to_string(), parent.to_string()))
            .collect();

        Self {
            builtins,
            builtin_subpaths,
            extensions: vec![
                ".js".to_string(),
                ".mjs".to_string(),
                ".cjs".to_string(),
                ".json".to_string(),
                ".node".to_string(),
            ],
        }
    }

    /// Get all built-in module names (for module.builtinModules)
    pub fn builtin_modules() -> Vec<&'static str> {
        BUILTIN_MODULES.to_vec()
    }

    /// Check if a specifier uses the node: prefix
    pub fn has_node_prefix(specifier: &str) -> bool {
        specifier.starts_with("node:")
    }

    /// Strip the node: prefix if present
    pub fn strip_node_prefix(specifier: &str) -> &str {
        specifier.strip_prefix("node:").unwrap_or(specifier)
    }

    /// Check if a module is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        // Handle node: prefix
        let name = Self::strip_node_prefix(name);

        // Check direct built-in
        if self.builtins.contains(&name.to_string()) {
            return true;
        }

        // Check subpath built-in (e.g., fs/promises)
        self.builtin_subpaths.contains_key(name)
    }

    /// Resolve a built-in module, handling subpaths
    pub fn resolve_builtin(&self, name: &str) -> Option<BuiltinResolveResult> {
        let name = Self::strip_node_prefix(name);

        // Check direct built-in
        if self.builtins.contains(&name.to_string()) {
            return Some(BuiltinResolveResult {
                module: name.to_string(),
                subpath: None,
            });
        }

        // Check subpath built-in
        if let Some(parent) = self.builtin_subpaths.get(name) {
            return Some(BuiltinResolveResult {
                module: parent.clone(),
                subpath: Some(name.to_string()),
            });
        }

        None
    }

    /// Resolve a module specifier
    pub fn resolve(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        // Check for node: prefix - these MUST be built-ins
        let has_node_prefix = Self::has_node_prefix(specifier);
        let bare_specifier = Self::strip_node_prefix(specifier);

        // Check if built-in module
        if let Some(builtin) = self.resolve_builtin(specifier) {
            return if let Some(subpath) = builtin.subpath {
                Ok(ResolveResult::BuiltInSubpath {
                    module: builtin.module,
                    subpath,
                })
            } else {
                Ok(ResolveResult::BuiltIn(builtin.module))
            };
        }

        // If it had node: prefix but isn't a built-in, error
        if has_node_prefix {
            return Err(NodeError::ModuleNotFound(format!(
                "Cannot find built-in module '{}'",
                specifier
            )));
        }

        // Check if relative or absolute path
        if bare_specifier.starts_with("./")
            || bare_specifier.starts_with("../")
            || bare_specifier.starts_with('/')
            || (cfg!(windows) && bare_specifier.chars().nth(1) == Some(':'))
        {
            return self.resolve_file(bare_specifier, parent_path);
        }

        // Otherwise, resolve as node_modules package
        self.resolve_node_modules(bare_specifier, parent_path)
    }

    /// Resolve a file path
    fn resolve_file(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
        let path = parent_dir.join(specifier);

        // Try exact path first
        if path.is_file() {
            return self.categorize_file(&path);
        }

        // Try with extensions
        for ext in &self.extensions {
            let with_ext = path.with_extension(ext.trim_start_matches('.'));
            if with_ext.is_file() {
                return self.categorize_file(&with_ext);
            }

            // Also try appending extension to full path
            let mut full_path = path.clone();
            let mut filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            filename.push_str(ext);
            full_path.set_file_name(&filename);
            if full_path.is_file() {
                return self.categorize_file(&full_path);
            }
        }

        // Try as directory with index file
        if path.is_dir() {
            return self.resolve_directory(&path);
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Resolve a directory (look for package.json main or index.js)
    fn resolve_directory(&self, dir: &Path) -> Result<ResolveResult> {
        self.resolve_directory_with_conditions(dir, &["default", "node", "require"])
    }

    /// Resolve a directory with specific conditions (for ESM vs CJS)
    pub fn resolve_directory_with_conditions(
        &self,
        dir: &Path,
        conditions: &[&str],
    ) -> Result<ResolveResult> {
        // Check for package.json
        let package_json_path = dir.join("package.json");
        if package_json_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                    // Try "exports" field first (modern resolution)
                    if let Some(exports) = &pkg.exports {
                        if let Some(resolved) = self.resolve_exports(dir, exports, ".", conditions)
                        {
                            return self.categorize_file(&resolved);
                        }
                    }

                    // Try "module" field for ESM
                    if conditions.contains(&"import") {
                        if let Some(module) = &pkg.module {
                            let module_path = dir.join(module);
                            if module_path.is_file() {
                                return self.categorize_file(&module_path);
                            }
                        }
                    }

                    // Try "main" field
                    if let Some(main) = &pkg.main {
                        let main_path = dir.join(main);
                        if main_path.is_file() {
                            return self.categorize_file(&main_path);
                        }
                        // Try with extensions
                        for ext in &self.extensions {
                            let with_ext = main_path.with_extension(ext.trim_start_matches('.'));
                            if with_ext.is_file() {
                                return self.categorize_file(&with_ext);
                            }
                        }
                    }
                }
            }
        }

        // Try index files
        for ext in &self.extensions {
            let index = dir.join(format!("index{}", ext));
            if index.is_file() {
                return self.categorize_file(&index);
            }
        }

        Err(NodeError::ModuleNotFound(dir.display().to_string()))
    }

    /// Resolve package.json "exports" field
    fn resolve_exports(
        &self,
        pkg_dir: &Path,
        exports: &serde_json::Value,
        subpath: &str,
        conditions: &[&str],
    ) -> Option<PathBuf> {
        match exports {
            // String shorthand: "exports": "./main.js"
            serde_json::Value::String(s) => {
                if subpath == "." {
                    Some(pkg_dir.join(s.trim_start_matches("./")))
                } else {
                    None
                }
            }

            // Array of fallbacks
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let Some(resolved) = self.resolve_exports(pkg_dir, item, subpath, conditions)
                    {
                        return Some(resolved);
                    }
                }
                None
            }

            // Object: conditional exports or subpath exports
            serde_json::Value::Object(obj) => {
                // Check if this is conditional exports (keys are conditions)
                let is_conditional = obj.keys().any(|k| {
                    matches!(
                        k.as_str(),
                        "import" | "require" | "node" | "default" | "browser" | "types"
                    )
                });

                if is_conditional {
                    // Resolve conditions in order
                    for condition in conditions {
                        if let Some(value) = obj.get(*condition) {
                            if let Some(resolved) =
                                self.resolve_exports(pkg_dir, value, subpath, conditions)
                            {
                                return Some(resolved);
                            }
                        }
                    }
                    // Try "default" last
                    if let Some(value) = obj.get("default") {
                        return self.resolve_exports(pkg_dir, value, subpath, conditions);
                    }
                } else {
                    // Subpath exports
                    // Try exact match first
                    if let Some(value) = obj.get(subpath) {
                        return self.resolve_exports(pkg_dir, value, ".", conditions);
                    }

                    // Try pattern matching (e.g., "./*": "./src/*.js")
                    for (pattern, value) in obj {
                        if pattern.contains('*') {
                            if let Some(matched) = self.match_subpath_pattern(subpath, pattern) {
                                if let serde_json::Value::String(target) = value {
                                    let resolved = target.replace('*', &matched);
                                    return Some(pkg_dir.join(resolved.trim_start_matches("./")));
                                }
                            }
                        }
                    }
                }
                None
            }

            _ => None,
        }
    }

    /// Match a subpath against a pattern with wildcards
    fn match_subpath_pattern(&self, subpath: &str, pattern: &str) -> Option<String> {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();
        if pattern_parts.len() != 2 {
            return None;
        }

        let prefix = pattern_parts[0];
        let suffix = pattern_parts[1];

        if subpath.starts_with(prefix) && subpath.ends_with(suffix) {
            let start = prefix.len();
            let end = subpath.len() - suffix.len();
            if start <= end {
                return Some(subpath[start..end].to_string());
            }
        }

        None
    }

    /// Resolve for ESM (import)
    pub fn resolve_esm(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        // Check for node: prefix - these MUST be built-ins
        let has_node_prefix = Self::has_node_prefix(specifier);
        let bare_specifier = Self::strip_node_prefix(specifier);

        // Check if built-in module
        if let Some(builtin) = self.resolve_builtin(specifier) {
            return if let Some(subpath) = builtin.subpath {
                Ok(ResolveResult::BuiltInSubpath {
                    module: builtin.module,
                    subpath,
                })
            } else {
                Ok(ResolveResult::BuiltIn(builtin.module))
            };
        }

        // If it had node: prefix but isn't a built-in, error
        if has_node_prefix {
            return Err(NodeError::ModuleNotFound(format!(
                "Cannot find built-in module '{}'",
                specifier
            )));
        }

        // Check if relative or absolute path
        if bare_specifier.starts_with("./")
            || bare_specifier.starts_with("../")
            || bare_specifier.starts_with('/')
            || (cfg!(windows) && bare_specifier.chars().nth(1) == Some(':'))
        {
            return self.resolve_file_esm(bare_specifier, parent_path);
        }

        // Otherwise, resolve as node_modules package with ESM conditions
        self.resolve_node_modules_esm(bare_specifier, parent_path)
    }

    /// Resolve a file path for ESM
    fn resolve_file_esm(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
        let path = parent_dir.join(specifier);

        // ESM requires explicit extensions for relative imports
        if path.is_file() {
            return self.categorize_file(&path);
        }

        // Try with extensions (less strict for compatibility)
        for ext in &self.extensions {
            let with_ext = path.with_extension(ext.trim_start_matches('.'));
            if with_ext.is_file() {
                return self.categorize_file(&with_ext);
            }
        }

        // Try as directory with index file
        if path.is_dir() {
            return self.resolve_directory_with_conditions(&path, &["import", "node", "default"]);
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Resolve a module from node_modules for ESM
    fn resolve_node_modules_esm(
        &self,
        specifier: &str,
        parent_path: &Path,
    ) -> Result<ResolveResult> {
        let (package_name, subpath) = self.parse_package_specifier(specifier);

        let mut current = parent_path.parent();
        while let Some(dir) = current {
            let node_modules = dir.join("node_modules").join(package_name);

            if node_modules.exists() {
                if let Some(sub) = subpath {
                    // Resolve subpath within package
                    let subpath_key = format!("./{}", sub);

                    // Check package.json exports
                    let pkg_json_path = node_modules.join("package.json");
                    if pkg_json_path.is_file() {
                        if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
                            if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                                if let Some(exports) = &pkg.exports {
                                    if let Some(resolved) = self.resolve_exports(
                                        &node_modules,
                                        exports,
                                        &subpath_key,
                                        &["import", "node", "default"],
                                    ) {
                                        return self.categorize_file(&resolved);
                                    }
                                }
                            }
                        }
                    }

                    // Fallback to direct path
                    let subpath_full = node_modules.join(sub);
                    if subpath_full.is_file() {
                        return self.categorize_file(&subpath_full);
                    }
                    for ext in &self.extensions {
                        let with_ext = subpath_full.with_extension(ext.trim_start_matches('.'));
                        if with_ext.is_file() {
                            return self.categorize_file(&with_ext);
                        }
                    }
                    if subpath_full.is_dir() {
                        return self.resolve_directory_with_conditions(
                            &subpath_full,
                            &["import", "node", "default"],
                        );
                    }
                } else {
                    // Resolve package main with ESM conditions
                    return self.resolve_directory_with_conditions(
                        &node_modules,
                        &["import", "node", "default"],
                    );
                }
            }

            current = dir.parent();
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Resolve a module from node_modules
    fn resolve_node_modules(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        // Split package name and subpath
        let (package_name, subpath) = self.parse_package_specifier(specifier);

        // Walk up directory tree looking for node_modules
        let mut current = parent_path.parent();
        while let Some(dir) = current {
            let node_modules = dir.join("node_modules").join(package_name);

            if node_modules.exists() {
                if let Some(sub) = subpath {
                    // Resolve subpath within package
                    let subpath_full = node_modules.join(sub);
                    if subpath_full.is_file() {
                        return self.categorize_file(&subpath_full);
                    }
                    for ext in &self.extensions {
                        let with_ext = subpath_full.with_extension(ext.trim_start_matches('.'));
                        if with_ext.is_file() {
                            return self.categorize_file(&with_ext);
                        }
                    }
                    if subpath_full.is_dir() {
                        return self.resolve_directory(&subpath_full);
                    }
                } else {
                    // Resolve package main
                    return self.resolve_directory(&node_modules);
                }
            }

            current = dir.parent();
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Parse a package specifier into name and optional subpath
    fn parse_package_specifier<'a>(&self, specifier: &'a str) -> (&'a str, Option<&'a str>) {
        if specifier.starts_with('@') {
            // Scoped package: @scope/name or @scope/name/subpath
            if let Some(slash_pos) = specifier[1..].find('/') {
                let after_scope = &specifier[slash_pos + 2..];
                if let Some(subpath_pos) = after_scope.find('/') {
                    let name_end = slash_pos + 2 + subpath_pos;
                    return (&specifier[..name_end], Some(&specifier[name_end + 1..]));
                }
            }
            (specifier, None)
        } else {
            // Regular package: name or name/subpath
            if let Some(slash_pos) = specifier.find('/') {
                (&specifier[..slash_pos], Some(&specifier[slash_pos + 1..]))
            } else {
                (specifier, None)
            }
        }
    }

    /// Categorize a file by extension
    fn categorize_file(&self, path: &Path) -> Result<ResolveResult> {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Ok(ResolveResult::Json(path)),
            Some("node") => Ok(ResolveResult::Native(path)),
            _ => Ok(ResolveResult::File(path)),
        }
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal package.json structure for resolution
#[derive(Debug, Deserialize)]
struct PackageJson {
    /// Main entry point (CommonJS)
    main: Option<String>,
    /// Module entry point (ESM, legacy)
    module: Option<String>,
    /// Exports map (modern)
    #[serde(default)]
    exports: Option<serde_json::Value>,
    /// Imports map (subpath imports)
    #[serde(default)]
    imports: Option<serde_json::Value>,
    /// Package type ("module" or "commonjs")
    #[serde(rename = "type")]
    type_field: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin() {
        let resolver = ModuleResolver::new();
        assert!(resolver.is_builtin("fs"));
        assert!(resolver.is_builtin("path"));
        assert!(resolver.is_builtin("node:fs"));
        assert!(resolver.is_builtin("node:path"));
        assert!(!resolver.is_builtin("lodash"));
        assert!(!resolver.is_builtin("node:lodash"));
    }

    #[test]
    fn test_is_builtin_subpath() {
        let resolver = ModuleResolver::new();
        assert!(resolver.is_builtin("fs/promises"));
        assert!(resolver.is_builtin("node:fs/promises"));
        assert!(resolver.is_builtin("dns/promises"));
        assert!(resolver.is_builtin("node:dns/promises"));
        assert!(resolver.is_builtin("stream/web"));
        assert!(resolver.is_builtin("node:stream/web"));
        assert!(resolver.is_builtin("util/types"));
        assert!(resolver.is_builtin("node:util/types"));
    }

    #[test]
    fn test_resolve_builtin() {
        let resolver = ModuleResolver::new();

        // Direct built-in
        let result = resolver.resolve_builtin("fs").unwrap();
        assert_eq!(result.module, "fs");
        assert!(result.subpath.is_none());

        // With node: prefix
        let result = resolver.resolve_builtin("node:fs").unwrap();
        assert_eq!(result.module, "fs");
        assert!(result.subpath.is_none());

        // Subpath built-in
        let result = resolver.resolve_builtin("fs/promises").unwrap();
        assert_eq!(result.module, "fs");
        assert_eq!(result.subpath, Some("fs/promises".to_string()));

        // Subpath with node: prefix
        let result = resolver.resolve_builtin("node:fs/promises").unwrap();
        assert_eq!(result.module, "fs");
        assert_eq!(result.subpath, Some("fs/promises".to_string()));

        // Non-builtin
        assert!(resolver.resolve_builtin("lodash").is_none());
    }

    #[test]
    fn test_has_node_prefix() {
        assert!(ModuleResolver::has_node_prefix("node:fs"));
        assert!(ModuleResolver::has_node_prefix("node:fs/promises"));
        assert!(!ModuleResolver::has_node_prefix("fs"));
        assert!(!ModuleResolver::has_node_prefix("./local"));
    }

    #[test]
    fn test_strip_node_prefix() {
        assert_eq!(ModuleResolver::strip_node_prefix("node:fs"), "fs");
        assert_eq!(
            ModuleResolver::strip_node_prefix("node:fs/promises"),
            "fs/promises"
        );
        assert_eq!(ModuleResolver::strip_node_prefix("fs"), "fs");
        assert_eq!(ModuleResolver::strip_node_prefix("./local"), "./local");
    }

    #[test]
    fn test_builtin_modules() {
        let modules = ModuleResolver::builtin_modules();
        assert!(modules.contains(&"fs"));
        assert!(modules.contains(&"path"));
        assert!(modules.contains(&"http"));
        assert!(modules.contains(&"test")); // Experimental
    }

    #[test]
    fn test_parse_package_specifier() {
        let resolver = ModuleResolver::new();

        assert_eq!(resolver.parse_package_specifier("lodash"), ("lodash", None));
        assert_eq!(
            resolver.parse_package_specifier("lodash/get"),
            ("lodash", Some("get"))
        );
        assert_eq!(
            resolver.parse_package_specifier("@types/node"),
            ("@types/node", None)
        );
        assert_eq!(
            resolver.parse_package_specifier("@babel/core/lib/index"),
            ("@babel/core", Some("lib/index"))
        );
    }
}
