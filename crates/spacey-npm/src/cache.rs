//! Package cache management.

use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

use crate::error::{Result, SnpmError};

/// Package cache for storing downloaded tarballs.
#[derive(Clone)]
pub struct PackageCache {
    cache_dir: PathBuf,
}

impl PackageCache {
    /// Create a new package cache.
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self> {
        let cache_dir = cache_dir.unwrap_or_else(default_cache_dir);

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the path to a cached tarball.
    pub fn get_tarball(&self, name: &str, version: &str) -> Option<PathBuf> {
        let tarball_path = self.tarball_path(name, version);
        if tarball_path.exists() {
            Some(tarball_path)
        } else {
            None
        }
    }

    /// Store a tarball in the cache.
    pub async fn store_tarball(&self, name: &str, version: &str, data: &[u8]) -> Result<PathBuf> {
        let tarball_path = self.tarball_path(name, version);

        // Create parent directory
        if let Some(parent) = tarball_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write tarball
        fs::write(&tarball_path, data).await?;
        debug!("Cached tarball at {}", tarball_path.display());

        Ok(tarball_path)
    }

    /// Get the path for a tarball.
    fn tarball_path(&self, name: &str, version: &str) -> PathBuf {
        // Handle scoped packages
        let safe_name = name.replace('/', "-").replace('@', "");
        self.cache_dir
            .join("tarballs")
            .join(format!("{}-{}.tgz", safe_name, version))
    }

    /// Get cached package metadata.
    pub fn get_metadata(&self, name: &str) -> Option<String> {
        let metadata_path = self.metadata_path(name);
        std::fs::read_to_string(metadata_path).ok()
    }

    /// Store package metadata.
    pub async fn store_metadata(&self, name: &str, metadata: &str) -> Result<()> {
        let metadata_path = self.metadata_path(name);

        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&metadata_path, metadata).await?;
        Ok(())
    }

    /// Get the path for metadata.
    fn metadata_path(&self, name: &str) -> PathBuf {
        let safe_name = name.replace('/', "-").replace('@', "");
        self.cache_dir
            .join("metadata")
            .join(format!("{}.json", safe_name))
    }

    /// Clear the entire cache.
    pub async fn clear(&self) -> Result<()> {
        info!("Clearing cache at {}", self.cache_dir.display());

        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).await?;
        }

        fs::create_dir_all(&self.cache_dir).await?;
        Ok(())
    }

    /// Get cache size in bytes.
    pub fn size(&self) -> Result<u64> {
        let mut total = 0u64;

        for entry in walkdir::WalkDir::new(&self.cache_dir) {
            let entry = entry.map_err(|e| SnpmError::Other(e.to_string()))?;
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }

        Ok(total)
    }

    /// List all cached packages.
    pub fn list(&self) -> Result<Vec<CachedPackage>> {
        let mut packages = Vec::new();
        let tarballs_dir = self.cache_dir.join("tarballs");

        if !tarballs_dir.exists() {
            return Ok(packages);
        }

        for entry in std::fs::read_dir(&tarballs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "tgz") {
                if let Some(file_name) = path.file_stem() {
                    let name = file_name.to_string_lossy();
                    // Parse name-version.tgz format
                    if let Some(last_dash) = name.rfind('-') {
                        let package_name = &name[..last_dash];
                        let version = &name[last_dash + 1..];

                        packages.push(CachedPackage {
                            name: package_name.to_string(),
                            version: version.to_string(),
                            path: path.clone(),
                            size: entry.metadata()?.len(),
                        });
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Verify cache integrity.
    pub async fn verify(&self) -> Result<VerifyResult> {
        let mut valid = 0;
        let mut invalid = 0;
        let mut missing = 0;

        let tarballs_dir = self.cache_dir.join("tarballs");

        if !tarballs_dir.exists() {
            return Ok(VerifyResult {
                valid,
                invalid,
                missing,
            });
        }

        for entry in std::fs::read_dir(&tarballs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "tgz") {
                if path.exists() {
                    // Basic check: try to read the file
                    match std::fs::metadata(&path) {
                        Ok(meta) if meta.len() > 0 => valid += 1,
                        _ => invalid += 1,
                    }
                } else {
                    missing += 1;
                }
            }
        }

        Ok(VerifyResult {
            valid,
            invalid,
            missing,
        })
    }
}

/// Default cache directory.
fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("snpm")
}

/// Information about a cached package.
#[derive(Debug, Clone)]
pub struct CachedPackage {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Path to the tarball
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
}

/// Result of cache verification.
#[derive(Debug)]
pub struct VerifyResult {
    /// Number of valid entries
    pub valid: usize,
    /// Number of invalid entries
    pub invalid: usize,
    /// Number of missing entries
    pub missing: usize,
}
