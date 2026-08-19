//! Package.json parsing and manipulation.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{Result, SnpmError};

/// Deserialize engines field which can be either an array or a map.
fn deserialize_engines<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EnginesField {
        Map(BTreeMap<String, String>),
        Array(Vec<String>),
    }

    match EnginesField::deserialize(deserializer)? {
        EnginesField::Map(map) => Ok(map),
        EnginesField::Array(arr) => {
            // Convert array to map with "*" as version requirement
            Ok(arr.into_iter().map(|k| (k, "*".to_string())).collect())
        }
    }
}

/// Represents a package.json file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    /// Package name
    pub name: Option<String>,

    /// Package version
    pub version: Option<String>,

    /// Package description
    pub description: Option<String>,

    /// Main entry point
    pub main: Option<String>,

    /// Module entry point (ES modules)
    pub module: Option<String>,

    /// TypeScript types entry point
    pub types: Option<String>,

    /// Package type (commonjs or module)
    #[serde(rename = "type")]
    pub package_type: Option<String>,

    /// Binary executables
    pub bin: Option<BinField>,

    /// Scripts
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,

    /// Production dependencies
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,

    /// Development dependencies
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: BTreeMap<String, String>,

    /// Peer dependencies
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: BTreeMap<String, String>,

    /// Peer dependency metadata
    #[serde(default, rename = "peerDependenciesMeta")]
    pub peer_dependencies_meta: BTreeMap<String, PeerDependencyMeta>,

    /// Optional dependencies
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: BTreeMap<String, String>,

    /// Bundled dependencies
    #[serde(default, rename = "bundledDependencies")]
    pub bundled_dependencies: Vec<String>,

    /// Package keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Package author
    pub author: Option<AuthorField>,

    /// Package license
    pub license: Option<String>,

    /// Repository information
    pub repository: Option<RepositoryField>,

    /// Bug tracking URL
    pub bugs: Option<BugsField>,

    /// Homepage URL
    pub homepage: Option<String>,

    /// Engines (Node.js version requirements)
    #[serde(default, deserialize_with = "deserialize_engines")]
    pub engines: BTreeMap<String, String>,

    /// Operating systems
    #[serde(default)]
    pub os: Vec<String>,

    /// CPU architectures
    #[serde(default)]
    pub cpu: Vec<String>,

    /// Private package flag
    pub private: Option<bool>,

    /// Publish configuration
    #[serde(rename = "publishConfig")]
    pub publish_config: Option<PublishConfig>,

    /// Workspaces
    pub workspaces: Option<WorkspacesField>,

    /// Files to include in package
    #[serde(default)]
    pub files: Vec<String>,

    /// Exports field (package exports)
    pub exports: Option<serde_json::Value>,

    /// Imports field (package imports)
    pub imports: Option<BTreeMap<String, serde_json::Value>>,

    /// Package overrides
    pub overrides: Option<BTreeMap<String, serde_json::Value>>,

    /// Resolution overrides
    pub resolutions: Option<BTreeMap<String, String>>,

    /// Additional fields not explicitly defined
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Binary field can be a string or a map
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinField {
    /// Single binary
    Single(String),
    /// Multiple binaries
    Multiple(BTreeMap<String, String>),
}

/// Author field can be a string or an object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthorField {
    /// Simple string format
    String(String),
    /// Object format
    Object {
        name: String,
        email: Option<String>,
        url: Option<String>,
    },
}

/// Repository field can be a string or an object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepositoryField {
    /// Simple string format
    String(String),
    /// Object format
    Object {
        #[serde(rename = "type")]
        repo_type: Option<String>,
        url: String,
        directory: Option<String>,
    },
}

/// Bugs field can be a string or an object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BugsField {
    /// Simple URL string
    String(String),
    /// Object format
    Object {
        url: Option<String>,
        email: Option<String>,
    },
}

/// Peer dependency metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerDependencyMeta {
    /// Whether this peer dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Publish configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    /// Registry to publish to
    pub registry: Option<String>,
    /// Access level (public or restricted)
    pub access: Option<String>,
    /// Tag to publish under
    pub tag: Option<String>,
}

/// Workspaces field can be an array or an object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspacesField {
    /// Simple array of globs
    Array(Vec<String>),
    /// Object with packages and nohoist
    Object {
        packages: Vec<String>,
        #[serde(default)]
        nohoist: Vec<String>,
    },
}

impl PackageJson {
    /// Read package.json from a file path.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SnpmError::PackageJsonNotFound(
                    path.as_ref()
                        .parent()
                        .unwrap_or(path.as_ref())
                        .display()
                        .to_string(),
                )
            } else {
                e.into()
            }
        })?;
        Self::parse(&content)
    }

    /// Parse package.json from a string.
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(|e| SnpmError::InvalidPackageJson(e.to_string()))
    }

    /// Write package.json to a file path.
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the package identifier (name@version).
    pub fn id(&self) -> String {
        match (&self.name, &self.version) {
            (Some(name), Some(version)) => format!("{}@{}", name, version),
            (Some(name), None) => name.clone(),
            _ => String::from("unnamed"),
        }
    }

    /// Get all dependencies (production + dev + optional).
    pub fn all_dependencies(&self) -> BTreeMap<String, String> {
        let mut deps = self.dependencies.clone();
        deps.extend(self.dev_dependencies.clone());
        deps.extend(self.optional_dependencies.clone());
        deps
    }

    /// Get production dependencies only.
    pub fn production_dependencies(&self) -> &BTreeMap<String, String> {
        &self.dependencies
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, name: &str, version: &str, dep_type: DependencyType) {
        match dep_type {
            DependencyType::Production => {
                self.dependencies
                    .insert(name.to_string(), version.to_string());
            }
            DependencyType::Development => {
                self.dev_dependencies
                    .insert(name.to_string(), version.to_string());
            }
            DependencyType::Peer => {
                self.peer_dependencies
                    .insert(name.to_string(), version.to_string());
            }
            DependencyType::Optional => {
                self.optional_dependencies
                    .insert(name.to_string(), version.to_string());
            }
        }
    }

    /// Remove a dependency.
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        let mut removed = false;
        removed |= self.dependencies.remove(name).is_some();
        removed |= self.dev_dependencies.remove(name).is_some();
        removed |= self.peer_dependencies.remove(name).is_some();
        removed |= self.optional_dependencies.remove(name).is_some();
        removed
    }

    /// Check if package has a script.
    pub fn has_script(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }

    /// Get a script command.
    pub fn get_script(&self, name: &str) -> Option<&String> {
        self.scripts.get(name)
    }

    /// Get workspace globs.
    pub fn workspace_globs(&self) -> Vec<String> {
        match &self.workspaces {
            Some(WorkspacesField::Array(globs)) => globs.clone(),
            Some(WorkspacesField::Object { packages, .. }) => packages.clone(),
            None => vec![],
        }
    }
}

/// Dependency type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    /// Production dependency
    Production,
    /// Development dependency
    Development,
    /// Peer dependency
    Peer,
    /// Optional dependency
    Optional,
}

/// Represents a resolved package from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPackage {
    /// Package name
    pub name: String,

    /// Available versions
    pub versions: BTreeMap<String, PackageVersion>,

    /// Distribution tags
    #[serde(rename = "dist-tags")]
    pub dist_tags: BTreeMap<String, String>,

    /// Last modified timestamp
    pub time: Option<BTreeMap<String, String>>,

    /// Package description
    pub description: Option<String>,

    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Author
    pub author: Option<AuthorField>,

    /// License
    pub license: Option<String>,

    /// Homepage
    pub homepage: Option<String>,

    /// Repository
    pub repository: Option<RepositoryField>,

    /// Bugs
    pub bugs: Option<BugsField>,

    /// Readme content
    pub readme: Option<String>,
}

/// Represents a specific version of a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageVersion {
    /// Package name
    pub name: String,

    /// Version string
    pub version: String,

    /// Description
    pub description: Option<String>,

    /// Main entry point
    pub main: Option<String>,

    /// Scripts
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,

    /// Dependencies
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,

    /// Dev dependencies
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, String>,

    /// Peer dependencies
    #[serde(default)]
    pub peer_dependencies: BTreeMap<String, String>,

    /// Optional dependencies
    #[serde(default)]
    pub optional_dependencies: BTreeMap<String, String>,

    /// Engines
    #[serde(default, deserialize_with = "deserialize_engines")]
    pub engines: BTreeMap<String, String>,

    /// Distribution information
    pub dist: PackageDist,

    /// Binary executables
    pub bin: Option<BinField>,

    /// Deprecated message
    pub deprecated: Option<String>,

    /// Has install scripts
    #[serde(default)]
    pub has_install_script: bool,
}

/// Package distribution information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDist {
    /// Integrity hash (usually sha512)
    pub integrity: Option<String>,

    /// SHA-1 hash (legacy)
    pub shasum: Option<String>,

    /// Tarball URL
    pub tarball: String,

    /// Number of files in the tarball
    #[serde(rename = "fileCount")]
    pub file_count: Option<u32>,

    /// Unpacked size in bytes
    #[serde(rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,

    /// NPM signature
    #[serde(rename = "npm-signature")]
    pub npm_signature: Option<String>,
}
