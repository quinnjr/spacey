// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Module cache for require()

use dashmap::DashMap;
use spacey_spidermonkey::Value;
use std::path::PathBuf;

/// Cached module entry
#[derive(Debug, Clone)]
pub struct CachedModule {
    /// The module's exports
    pub exports: Value,
    /// The module's filename
    pub filename: PathBuf,
    /// Whether the module has finished loading
    pub loaded: bool,
    /// Child modules required by this module
    pub children: Vec<PathBuf>,
    /// Parent module that required this one
    pub parent: Option<PathBuf>,
}

/// Thread-safe module cache
pub struct ModuleCache {
    /// Cache mapping absolute paths to cached modules
    cache: DashMap<PathBuf, CachedModule>,
}

impl ModuleCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get a cached module by path
    pub fn get(&self, path: &PathBuf) -> Option<CachedModule> {
        self.cache.get(path).map(|entry| entry.clone())
    }

    /// Check if a module is cached
    pub fn has(&self, path: &PathBuf) -> bool {
        self.cache.contains_key(path)
    }

    /// Add a module to the cache
    pub fn set(&self, path: PathBuf, module: CachedModule) {
        self.cache.insert(path, module);
    }

    /// Remove a module from the cache
    pub fn delete(&self, path: &PathBuf) -> Option<CachedModule> {
        self.cache.remove(path).map(|(_, v)| v)
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get all cached module paths
    pub fn keys(&self) -> Vec<PathBuf> {
        self.cache.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get the number of cached modules
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new()
    }
}
