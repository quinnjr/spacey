//! Error types for snpm.

use thiserror::Error;

/// Result type for snpm operations.
pub type Result<T> = std::result::Result<T, SnpmError>;

/// Main error type for snpm.
#[derive(Error, Debug)]
pub enum SnpmError {
    /// Package not found in registry
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    /// Version not found for package
    #[error("Version {version} not found for package {package}")]
    VersionNotFound { package: String, version: String },

    /// Invalid package.json
    #[error("Invalid package.json: {0}")]
    InvalidPackageJson(String),

    /// Invalid package-lock.json
    #[error("Invalid package-lock.json: {0}")]
    InvalidLockfile(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Semver parsing error
    #[error("Invalid version: {0}")]
    Semver(#[from] semver::Error),

    /// Integrity check failed
    #[error("Integrity check failed for {package}: expected {expected}, got {actual}")]
    IntegrityMismatch {
        package: String,
        expected: String,
        actual: String,
    },

    /// Dependency resolution error
    #[error("Failed to resolve dependencies: {0}")]
    ResolutionError(String),

    /// Script execution error
    #[error("Script '{script}' failed with exit code {code}")]
    ScriptFailed { script: String, code: i32 },

    /// Script not found
    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    /// Package.json not found
    #[error("package.json not found in {0}")]
    PackageJsonNotFound(String),

    /// Registry error
    #[error("Registry error: {0}")]
    Registry(String),

    /// Authentication error
    #[error("Authentication required for {0}")]
    AuthRequired(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Peer dependency conflict
    #[error("Peer dependency conflict: {0}")]
    PeerConflict(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// General error with message
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for SnpmError {
    fn from(err: anyhow::Error) -> Self {
        SnpmError::Other(err.to_string())
    }
}

impl From<&str> for SnpmError {
    fn from(s: &str) -> Self {
        SnpmError::Other(s.to_string())
    }
}

impl From<String> for SnpmError {
    fn from(s: String) -> Self {
        SnpmError::Other(s)
    }
}
