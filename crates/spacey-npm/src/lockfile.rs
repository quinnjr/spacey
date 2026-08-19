//! Package lock file management.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{Result, SnpmError};

/// Package lock file (package-lock.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLock {
    /// Lock file name
    pub name: String,

    /// Lock file version
    pub version: String,

    /// Lock file format version
    pub lockfile_version: u32,

    /// Whether to require lock file
    #[serde(default)]
    pub requires: bool,

    /// Packages (lockfileVersion >= 2)
    #[serde(default)]
    pub packages: BTreeMap<String, LockPackage>,

    /// Dependencies (lockfileVersion 1)
    #[serde(default)]
    pub dependencies: BTreeMap<String, LockDependency>,
}

/// A package entry in the lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockPackage {
    /// Package version
    pub version: String,

    /// Resolved URL
    pub resolved: Option<String>,

    /// Integrity hash
    pub integrity: Option<String>,

    /// Whether this is the root project
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub link: bool,

    /// Dependencies
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,

    /// Dev dependencies (only for root)
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, String>,

    /// Peer dependencies
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, String>,

    /// Peer dependencies meta
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies_meta: BTreeMap<String, PeerDependencyMeta>,

    /// Optional dependencies
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub optional_dependencies: BTreeMap<String, String>,

    /// Engines
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engines: BTreeMap<String, String>,

    /// Whether this is a dev dependency
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dev: bool,

    /// Whether this is optional
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,

    /// Whether this is a peer dependency
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub peer: bool,

    /// Whether this has install scripts
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_install_script: bool,

    /// Binary executables
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<BTreeMap<String, String>>,

    /// License
    pub license: Option<String>,
}

/// A dependency entry in lockfile v1 format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockDependency {
    /// Version
    pub version: String,

    /// Resolved URL
    pub resolved: Option<String>,

    /// Integrity hash
    pub integrity: Option<String>,

    /// Whether this is a dev dependency
    #[serde(default)]
    pub dev: bool,

    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,

    /// Nested dependencies
    #[serde(default)]
    pub dependencies: BTreeMap<String, LockDependency>,

    /// Required packages
    #[serde(default)]
    pub requires: BTreeMap<String, String>,
}

/// Peer dependency metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerDependencyMeta {
    /// Whether this peer dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl PackageLock {
    /// Create a new lock file.
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            lockfile_version: 3,
            requires: true,
            packages: BTreeMap::new(),
            dependencies: BTreeMap::new(),
        }
    }

    /// Read a lock file from disk.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&content)
    }

    /// Parse a lock file from a string.
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(|e| SnpmError::InvalidLockfile(e.to_string()))
    }

    /// Write the lock file to disk.
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a package to the lock file.
    pub fn add_package(&mut self, path: &str, package: LockPackage) {
        self.packages.insert(path.to_string(), package);
    }

    /// Get a package from the lock file.
    pub fn get_package(&self, path: &str) -> Option<&LockPackage> {
        self.packages.get(path)
    }

    /// Remove a package from the lock file.
    pub fn remove_package(&mut self, path: &str) -> Option<LockPackage> {
        self.packages.remove(path)
    }

    /// Check if a package is locked.
    pub fn is_locked(&self, name: &str, version: &str) -> bool {
        let path = format!("node_modules/{}", name);
        self.packages
            .get(&path)
            .map_or(false, |p| p.version == version)
    }

    /// Get all locked package names.
    pub fn locked_packages(&self) -> Vec<(&String, &LockPackage)> {
        self.packages
            .iter()
            .filter(|(path, _)| path.starts_with("node_modules/"))
            .collect()
    }

    /// Get resolved version for a package.
    pub fn get_resolved_version(&self, name: &str) -> Option<&str> {
        let path = format!("node_modules/{}", name);
        self.packages.get(&path).map(|p| p.version.as_str())
    }

    /// Update from v1 format to v3 format.
    pub fn upgrade_to_v3(&mut self) {
        if self.lockfile_version < 3 {
            // Convert dependencies to packages format
            for (name, dep) in &self.dependencies {
                let path = format!("node_modules/{}", name);
                let package = LockPackage {
                    version: dep.version.clone(),
                    resolved: dep.resolved.clone(),
                    integrity: dep.integrity.clone(),
                    dev: dep.dev,
                    optional: dep.optional,
                    dependencies: dep.requires.clone(),
                    ..Default::default()
                };
                self.packages.insert(path, package);
            }
            self.lockfile_version = 3;
        }
    }
}

impl Default for LockPackage {
    fn default() -> Self {
        Self {
            version: String::new(),
            resolved: None,
            integrity: None,
            link: false,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            peer_dependencies_meta: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            engines: BTreeMap::new(),
            dev: false,
            optional: false,
            peer: false,
            has_install_script: false,
            bin: None,
            license: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_lockfile() {
        let lock = PackageLock::new("test".into(), "1.0.0".into());
        assert_eq!(lock.lockfile_version, 3);
        assert!(lock.packages.is_empty());
    }

    #[test]
    fn test_add_package() {
        let mut lock = PackageLock::new("test".into(), "1.0.0".into());

        lock.add_package(
            "node_modules/lodash",
            LockPackage {
                version: "4.17.21".into(),
                resolved: Some("https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".into()),
                integrity: Some("sha512-xyz".into()),
                ..Default::default()
            },
        );

        assert!(lock.is_locked("lodash", "4.17.21"));
        assert!(!lock.is_locked("lodash", "4.17.20"));
    }
}
