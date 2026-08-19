// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! ECMAScript Modules (ESM) implementation
//!
//! Implements ES6+ module syntax:
//! - `import` declarations
//! - `export` declarations
//! - Dynamic `import()`
//! - `import.meta`

use crate::error::{NodeError, Result};
use crate::module_system::resolver::{ModuleResolver, ResolveResult};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::Deserialize;
use spacey_spidermonkey::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Module type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    /// CommonJS module (require/module.exports)
    CommonJS,
    /// ECMAScript module (import/export)
    ESM,
    /// JSON file
    Json,
    /// Unknown/detect from content
    Unknown,
}

impl ModuleType {
    /// Detect module type from file path
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("mjs") => ModuleType::ESM,
            Some("cjs") => ModuleType::CommonJS,
            Some("json") => ModuleType::Json,
            _ => ModuleType::Unknown,
        }
    }

    /// Detect module type from package.json "type" field
    pub fn from_package_type(type_field: Option<&str>) -> Self {
        match type_field {
            Some("module") => ModuleType::ESM,
            Some("commonjs") => ModuleType::CommonJS,
            _ => ModuleType::CommonJS, // Default to CommonJS
        }
    }
}

/// Import specifier types
#[derive(Debug, Clone)]
pub enum ImportSpecifier {
    /// `import foo from 'module'`
    Default(String),
    /// `import { foo } from 'module'`
    Named(String, Option<String>), // (imported, local alias)
    /// `import * as foo from 'module'`
    Namespace(String),
}

/// Export specifier types
#[derive(Debug, Clone)]
pub enum ExportSpecifier {
    /// `export default value`
    Default,
    /// `export { foo }` or `export { foo as bar }`
    Named(String, Option<String>), // (local, exported alias)
    /// `export * from 'module'`
    All(String), // from module
    /// `export * as name from 'module'`
    AllAs(String, String), // (name, from module)
}

/// Parsed import statement
#[derive(Debug, Clone)]
pub struct ImportDeclaration {
    /// The module specifier (e.g., './foo.js', 'lodash')
    pub specifier: String,
    /// Import specifiers
    pub imports: Vec<ImportSpecifier>,
    /// Whether this is a side-effect only import (`import 'module'`)
    pub side_effect_only: bool,
}

/// Parsed export statement
#[derive(Debug, Clone)]
pub struct ExportDeclaration {
    /// Export specifiers
    pub exports: Vec<ExportSpecifier>,
    /// Source module for re-exports
    pub from_module: Option<String>,
}

/// ESM module record
#[derive(Debug, Clone)]
pub struct EsmModule {
    /// Module URL/path
    pub url: PathBuf,
    /// Module namespace object (exports)
    pub namespace: HashMap<String, Value>,
    /// Whether the module has been evaluated
    pub evaluated: bool,
    /// Import dependencies
    pub dependencies: Vec<String>,
    /// Module type
    pub module_type: ModuleType,
}

impl EsmModule {
    /// Create a new ESM module
    pub fn new(url: PathBuf) -> Self {
        Self {
            url,
            namespace: HashMap::new(),
            evaluated: false,
            dependencies: Vec::new(),
            module_type: ModuleType::ESM,
        }
    }

    /// Get an exported value
    pub fn get_export(&self, name: &str) -> Option<&Value> {
        self.namespace.get(name)
    }

    /// Set an exported value
    pub fn set_export(&mut self, name: String, value: Value) {
        self.namespace.insert(name, value);
    }

    /// Get the default export
    pub fn get_default(&self) -> Option<&Value> {
        self.namespace.get("default")
    }

    /// Get all exports as a namespace object
    pub fn get_namespace(&self) -> Value {
        Value::NativeObject(self.namespace.clone())
    }
}

/// import.meta object
#[derive(Debug, Clone)]
pub struct ImportMeta {
    /// The URL of the current module
    pub url: String,
    /// The directory of the current module
    pub dirname: String,
    /// The filename of the current module
    pub filename: String,
    /// Whether this is the main module
    pub main: bool,
    /// resolve function reference
    pub resolve: Option<Value>,
}

impl ImportMeta {
    /// Create import.meta for a module
    pub fn new(module_path: &Path, is_main: bool) -> Self {
        let url = format!("file://{}", module_path.display());
        let dirname = module_path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let filename = module_path.display().to_string();

        Self {
            url,
            dirname,
            filename,
            main: is_main,
            resolve: None,
        }
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("url".to_string(), Value::String(self.url.clone()));
        obj.insert("dirname".to_string(), Value::String(self.dirname.clone()));
        obj.insert("filename".to_string(), Value::String(self.filename.clone()));
        obj.insert("main".to_string(), Value::Boolean(self.main));
        // resolve would be a function
        Value::NativeObject(obj)
    }
}

/// ESM module loader
pub struct EsmLoader {
    /// Module cache
    cache: DashMap<PathBuf, Arc<RwLock<EsmModule>>>,
    /// Module resolver
    resolver: ModuleResolver,
    /// Currently loading modules (for circular dependency detection)
    loading: DashMap<PathBuf, ()>,
    /// Main module path
    main_module: RwLock<Option<PathBuf>>,
}

impl EsmLoader {
    /// Create a new ESM loader
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            resolver: ModuleResolver::new(),
            loading: DashMap::new(),
            main_module: RwLock::new(None),
        }
    }

    /// Set the main module
    pub fn set_main_module(&self, path: PathBuf) {
        *self.main_module.write() = Some(path);
    }

    /// Check if a path is the main module
    pub fn is_main_module(&self, path: &Path) -> bool {
        self.main_module
            .read()
            .as_ref()
            .map(|p| p == path)
            .unwrap_or(false)
    }

    /// Resolve a module specifier
    pub fn resolve(&self, specifier: &str, parent: &Path) -> Result<PathBuf> {
        let resolved = self.resolver.resolve(specifier, parent)?;

        match resolved {
            ResolveResult::BuiltIn(name) => Err(NodeError::ModuleResolution {
                module: specifier.to_string(),
                reason: format!("Built-in module '{}' cannot be imported as ESM yet", name),
            }),
            ResolveResult::BuiltInSubpath { module, subpath } => Err(NodeError::ModuleResolution {
                module: specifier.to_string(),
                reason: format!(
                    "Built-in module '{}' (subpath '{}') cannot be imported as ESM yet",
                    module, subpath
                ),
            }),
            ResolveResult::File(path) | ResolveResult::Json(path) => Ok(path),
            ResolveResult::Native(path) => Err(NodeError::ModuleResolution {
                module: path.display().to_string(),
                reason: "Native modules (.node) cannot be imported as ESM".to_string(),
            }),
        }
    }

    /// Resolve a module specifier (returns ResolveResult for built-ins)
    pub fn resolve_with_builtins(&self, specifier: &str, parent: &Path) -> Result<ResolveResult> {
        self.resolver.resolve(specifier, parent)
    }

    /// Check if a specifier is a built-in module
    pub fn is_builtin(&self, specifier: &str) -> bool {
        self.resolver.is_builtin(specifier)
    }

    /// Determine module type for a file
    pub fn get_module_type(&self, path: &Path) -> Result<ModuleType> {
        // Check file extension first
        let ext_type = ModuleType::from_path(path);
        if ext_type != ModuleType::Unknown {
            return Ok(ext_type);
        }

        // Check package.json "type" field
        if let Some(pkg_type) = self.find_package_type(path)? {
            return Ok(pkg_type);
        }

        // Default to CommonJS for .js files
        Ok(ModuleType::CommonJS)
    }

    /// Find the package.json type field for a file
    fn find_package_type(&self, path: &Path) -> Result<Option<ModuleType>> {
        let mut current = path.parent();

        while let Some(dir) = current {
            let pkg_path = dir.join("package.json");
            if pkg_path.exists() {
                let content = std::fs::read_to_string(&pkg_path)?;
                if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                    return Ok(Some(ModuleType::from_package_type(
                        pkg.type_field.as_deref(),
                    )));
                }
            }
            current = dir.parent();
        }

        Ok(None)
    }

    /// Load a module
    pub fn load<'a>(
        &'a self,
        specifier: &'a str,
        parent: &'a Path,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Arc<RwLock<EsmModule>>>> + Send + 'a>,
    > {
        Box::pin(async move {
            let path = self.resolve(specifier, parent)?;
            self.load_module(&path).await
        })
    }

    /// Load a module by path
    pub fn load_module<'a>(
        &'a self,
        path: &'a Path,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Arc<RwLock<EsmModule>>>> + Send + 'a>,
    > {
        Box::pin(async move {
            let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

            // Check cache
            if let Some(module) = self.cache.get(&abs_path) {
                return Ok(Arc::clone(module.value()));
            }

            // Check for circular dependency
            if self.loading.contains_key(&abs_path) {
                // Return cached module for circular dependencies (may be partially evaluated)
                if let Some(module) = self.cache.get(&abs_path) {
                    return Ok(Arc::clone(module.value()));
                }
                return Err(NodeError::CircularDependency(
                    abs_path.display().to_string(),
                ));
            }

            // Mark as loading
            self.loading.insert(abs_path.clone(), ());

            // Create module record
            let module = Arc::new(RwLock::new(EsmModule::new(abs_path.clone())));
            self.cache.insert(abs_path.clone(), Arc::clone(&module));

            // Load and parse the module
            let source = std::fs::read_to_string(&abs_path)?;

            // Parse imports/exports (simplified - real implementation would use the parser)
            let (imports, _exports) = self.parse_module_syntax(&source)?;

            // Update module with dependencies
            {
                let mut mod_write = module.write();
                mod_write.dependencies = imports.iter().map(|i| i.specifier.clone()).collect();
            }

            // Load dependencies
            for import in &imports {
                let _dep = self.load(&import.specifier, &abs_path).await?;
            }

            // Mark loading complete
            self.loading.remove(&abs_path);

            Ok(module)
        })
    }

    /// Parse module syntax to extract imports and exports
    /// This is a simplified implementation - real version would use the AST
    fn parse_module_syntax(
        &self,
        source: &str,
    ) -> Result<(Vec<ImportDeclaration>, Vec<ExportDeclaration>)> {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        // Simple regex-based parsing (production would use proper AST)
        let import_re = regex::Regex::new(
            r#"import\s+(?:(?:(\w+)\s*,?\s*)?(?:\{\s*([^}]*)\s*\})?(?:\*\s+as\s+(\w+))?)\s+from\s+['"]([^'"]+)['"]"#
        ).unwrap();

        let import_side_effect_re = regex::Regex::new(r#"import\s+['"]([^'"]+)['"]"#).unwrap();

        let export_default_re = regex::Regex::new(r#"export\s+default\s+"#).unwrap();

        let export_named_re = regex::Regex::new(r#"export\s+\{\s*([^}]*)\s*\}"#).unwrap();

        let export_re_export_re =
            regex::Regex::new(r#"export\s+\{\s*([^}]*)\s*\}\s+from\s+['"]([^'"]+)['"]"#).unwrap();

        let export_all_re = regex::Regex::new(r#"export\s+\*\s+from\s+['"]([^'"]+)['"]"#).unwrap();

        let export_all_as_re =
            regex::Regex::new(r#"export\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]"#).unwrap();

        // Parse imports
        for cap in import_re.captures_iter(source) {
            let specifier = cap
                .get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let mut import_specs = Vec::new();

            // Default import
            if let Some(default) = cap.get(1) {
                import_specs.push(ImportSpecifier::Default(default.as_str().to_string()));
            }

            // Named imports
            if let Some(named) = cap.get(2) {
                for part in named.as_str().split(',') {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    if part.contains(" as ") {
                        let parts: Vec<&str> = part.split(" as ").collect();
                        if parts.len() == 2 {
                            import_specs.push(ImportSpecifier::Named(
                                parts[0].trim().to_string(),
                                Some(parts[1].trim().to_string()),
                            ));
                        }
                    } else {
                        import_specs.push(ImportSpecifier::Named(part.to_string(), None));
                    }
                }
            }

            // Namespace import
            if let Some(ns) = cap.get(3) {
                import_specs.push(ImportSpecifier::Namespace(ns.as_str().to_string()));
            }

            imports.push(ImportDeclaration {
                specifier,
                imports: import_specs,
                side_effect_only: false,
            });
        }

        // Parse side-effect only imports
        for cap in import_side_effect_re.captures_iter(source) {
            let specifier = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            // Skip if already matched as regular import
            if !imports.iter().any(|i| i.specifier == specifier) {
                imports.push(ImportDeclaration {
                    specifier,
                    imports: Vec::new(),
                    side_effect_only: true,
                });
            }
        }

        // Parse exports
        if export_default_re.is_match(source) {
            exports.push(ExportDeclaration {
                exports: vec![ExportSpecifier::Default],
                from_module: None,
            });
        }

        // Export * as name from 'module'
        for cap in export_all_as_re.captures_iter(source) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let from = cap
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            exports.push(ExportDeclaration {
                exports: vec![ExportSpecifier::AllAs(name, from.clone())],
                from_module: Some(from),
            });
        }

        // Export * from 'module'
        for cap in export_all_re.captures_iter(source) {
            let from = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            exports.push(ExportDeclaration {
                exports: vec![ExportSpecifier::All(from.clone())],
                from_module: Some(from),
            });
        }

        // Re-exports: export { ... } from 'module'
        for cap in export_re_export_re.captures_iter(source) {
            let named = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let from = cap
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let mut export_specs = Vec::new();
            for part in named.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if part.contains(" as ") {
                    let parts: Vec<&str> = part.split(" as ").collect();
                    if parts.len() == 2 {
                        export_specs.push(ExportSpecifier::Named(
                            parts[0].trim().to_string(),
                            Some(parts[1].trim().to_string()),
                        ));
                    }
                } else {
                    export_specs.push(ExportSpecifier::Named(part.to_string(), None));
                }
            }

            exports.push(ExportDeclaration {
                exports: export_specs,
                from_module: Some(from),
            });
        }

        // Named exports: export { ... }
        for cap in export_named_re.captures_iter(source) {
            // Skip re-exports (already handled)
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or_default();
            if full_match.contains(" from ") {
                continue;
            }

            let named = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let mut export_specs = Vec::new();

            for part in named.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if part.contains(" as ") {
                    let parts: Vec<&str> = part.split(" as ").collect();
                    if parts.len() == 2 {
                        export_specs.push(ExportSpecifier::Named(
                            parts[0].trim().to_string(),
                            Some(parts[1].trim().to_string()),
                        ));
                    }
                } else {
                    export_specs.push(ExportSpecifier::Named(part.to_string(), None));
                }
            }

            exports.push(ExportDeclaration {
                exports: export_specs,
                from_module: None,
            });
        }

        Ok((imports, exports))
    }

    /// Get import.meta for a module
    pub fn get_import_meta(&self, path: &Path) -> ImportMeta {
        ImportMeta::new(path, self.is_main_module(path))
    }

    /// Clear the module cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Check if a module is cached
    pub fn is_cached(&self, path: &Path) -> bool {
        self.cache.contains_key(path)
    }

    /// Get a cached module
    pub fn get_cached(&self, path: &Path) -> Option<Arc<RwLock<EsmModule>>> {
        self.cache.get(path).map(|m| Arc::clone(m.value()))
    }
}

impl Default for EsmLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal package.json structure for type detection
#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(rename = "type")]
    type_field: Option<String>,
    #[serde(default)]
    exports: Option<serde_json::Value>,
    #[serde(default)]
    imports: Option<serde_json::Value>,
}

/// Dynamic import function
pub async fn dynamic_import(loader: &EsmLoader, specifier: &str, parent: &Path) -> Result<Value> {
    let module = loader.load(specifier, parent).await?;
    let module_read = module.read();
    Ok(module_read.get_namespace())
}

/// Evaluate an ESM module
pub fn evaluate_module(_module: &mut EsmModule, _source: &str) -> Result<()> {
    // This would compile and execute the module using the engine
    // For now, just mark as evaluated
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_type_from_path() {
        assert_eq!(ModuleType::from_path(Path::new("foo.mjs")), ModuleType::ESM);
        assert_eq!(
            ModuleType::from_path(Path::new("foo.cjs")),
            ModuleType::CommonJS
        );
        assert_eq!(
            ModuleType::from_path(Path::new("foo.json")),
            ModuleType::Json
        );
        assert_eq!(
            ModuleType::from_path(Path::new("foo.js")),
            ModuleType::Unknown
        );
    }

    #[test]
    fn test_module_type_from_package() {
        assert_eq!(
            ModuleType::from_package_type(Some("module")),
            ModuleType::ESM
        );
        assert_eq!(
            ModuleType::from_package_type(Some("commonjs")),
            ModuleType::CommonJS
        );
        assert_eq!(ModuleType::from_package_type(None), ModuleType::CommonJS);
    }

    #[test]
    fn test_import_meta() {
        let meta = ImportMeta::new(Path::new("/home/user/project/main.js"), true);
        assert_eq!(meta.url, "file:///home/user/project/main.js");
        assert_eq!(meta.dirname, "/home/user/project");
        assert_eq!(meta.filename, "/home/user/project/main.js");
        assert!(meta.main);
    }

    #[test]
    fn test_parse_imports() {
        let loader = EsmLoader::new();

        let source = r#"
            import foo from 'foo';
            import { bar, baz as qux } from 'bar';
            import * as all from 'all';
            import 'side-effect';
        "#;

        let (imports, _) = loader.parse_module_syntax(source).unwrap();

        assert_eq!(imports.len(), 4);
        assert_eq!(imports[0].specifier, "foo");
        assert_eq!(imports[1].specifier, "bar");
        assert_eq!(imports[2].specifier, "all");
        assert_eq!(imports[3].specifier, "side-effect");
    }

    #[test]
    fn test_parse_exports() {
        let loader = EsmLoader::new();

        let source = r#"
            export default function() {}
            export { foo, bar as baz };
            export * from 'reexport';
            export * as ns from 'namespace';
        "#;

        let (_, exports) = loader.parse_module_syntax(source).unwrap();

        assert!(!exports.is_empty());
    }
}
