//! Content-addressable store for packages (pnpm-style).
//!
//! The store uses a content-addressable layout where packages are stored by their
//! integrity hash. This allows for:
//! - Deduplication across projects
//! - Hard links or symlinks instead of copies
//! - Efficient disk usage
//!
//! Store Layout:
//! ```text
//! ~/.snpm-store/
//! ├── v3/
//! │   └── files/
//! │       ├── 00/
//! │       │   └── <hash-prefix>/
//! │       │       └── <full-hash>/
//! │       │           └── node_modules/
//! │       │               └── <package-name>/
//! │       │                   └── <files...>
//! │       ├── 01/
//! │       └── ...
//! └── tmp/
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use sha2::{Digest, Sha512};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::error::{Result, SnpmError};
use crate::integrity::IntegrityChecker;

/// Store version for compatibility
const STORE_VERSION: &str = "v3";

/// Content-addressable package store.
#[derive(Clone)]
pub struct PackageStore {
    /// Root directory of the store
    store_dir: PathBuf,
    /// Index of packages in store (integrity -> store path)
    index: Arc<DashMap<String, StoredPackage>>,
    /// Integrity checker
    integrity: IntegrityChecker,
    /// Whether to use hard links (true) or symlinks (false)
    use_hard_links: bool,
}

/// Information about a stored package.
#[derive(Debug, Clone)]
pub struct StoredPackage {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Integrity hash
    pub integrity: String,
    /// Path in store
    pub store_path: PathBuf,
    /// Size in bytes
    pub size: u64,
}

/// Store statistics.
#[derive(Debug, Default)]
pub struct StoreStats {
    /// Total number of packages
    pub package_count: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Number of unique packages (by name)
    pub unique_packages: usize,
    /// Space saved by deduplication (estimated)
    pub space_saved: u64,
}

impl PackageStore {
    /// Create a new package store.
    pub fn new(store_dir: Option<PathBuf>) -> Result<Self> {
        let store_dir = store_dir.unwrap_or_else(default_store_dir);

        // Create store directories
        std::fs::create_dir_all(store_dir.join(STORE_VERSION).join("files"))?;
        std::fs::create_dir_all(store_dir.join("tmp"))?;

        let store = Self {
            store_dir,
            index: Arc::new(DashMap::new()),
            integrity: IntegrityChecker::new(),
            use_hard_links: true,
        };

        // Load existing index
        store.load_index()?;

        Ok(store)
    }

    /// Get the store directory.
    pub fn store_dir(&self) -> &Path {
        &self.store_dir
    }

    /// Check if a package is in the store.
    pub fn has_package(&self, integrity: &str) -> bool {
        if let Some(stored) = self.index.get(integrity) {
            stored.store_path.exists()
        } else {
            false
        }
    }

    /// Get a package from the store.
    pub fn get_package(&self, integrity: &str) -> Option<StoredPackage> {
        self.index.get(integrity).map(|r| r.clone())
    }

    /// Get the path to a package in the store.
    pub fn get_package_path(&self, integrity: &str) -> Option<PathBuf> {
        self.index.get(integrity).map(|r| r.store_path.clone())
    }

    /// Import a package tarball into the store.
    pub async fn import_package(
        &self,
        name: &str,
        version: &str,
        tarball_data: &[u8],
        integrity: Option<&str>,
    ) -> Result<StoredPackage> {
        // Calculate integrity if not provided
        let integrity_hash = integrity
            .map(String::from)
            .unwrap_or_else(|| self.integrity.compute_integrity(tarball_data));

        // Check if already in store
        if let Some(existing) = self.get_package(&integrity_hash) {
            debug!("Package {}@{} already in store", name, version);
            return Ok(existing);
        }

        // Calculate store path
        let store_path = self.calculate_store_path(&integrity_hash, name);

        // Create temporary extraction directory
        let tmp_dir = self.store_dir.join("tmp").join(format!(
            "{}-{}-{}",
            name.replace('/', "-"),
            version,
            std::process::id()
        ));
        fs::create_dir_all(&tmp_dir).await?;

        // Extract tarball
        let tarball_path = tmp_dir.join("package.tgz");
        fs::write(&tarball_path, tarball_data).await?;

        extract_tarball_async(&tarball_path, &tmp_dir).await?;

        // Move extracted content to store
        let package_dir = tmp_dir.join("package");
        if package_dir.exists() {
            fs::create_dir_all(store_path.parent().unwrap()).await?;

            // Remove existing if present (shouldn't happen but handle it)
            if store_path.exists() {
                fs::remove_dir_all(&store_path).await?;
            }

            fs::rename(&package_dir, &store_path).await?;
        }

        // Calculate size
        let size = calculate_dir_size(&store_path)?;

        // Cleanup temp directory
        let _ = fs::remove_dir_all(&tmp_dir).await;

        // Create stored package entry
        let stored = StoredPackage {
            name: name.to_string(),
            version: version.to_string(),
            integrity: integrity_hash.clone(),
            store_path: store_path.clone(),
            size,
        };

        // Add to index
        self.index.insert(integrity_hash, stored.clone());

        debug!(
            "Imported {}@{} to store at {}",
            name,
            version,
            store_path.display()
        );

        Ok(stored)
    }

    /// Link a package from the store to a target directory.
    pub async fn link_package(&self, integrity: &str, target_dir: &Path) -> Result<()> {
        let stored = self.get_package(integrity).ok_or_else(|| {
            SnpmError::Other(format!(
                "Package with integrity {} not found in store",
                integrity
            ))
        })?;

        // Create target parent directory
        if let Some(parent) = target_dir.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Remove existing target if present
        if target_dir.exists() {
            if target_dir.is_symlink() {
                fs::remove_file(target_dir).await?;
            } else {
                fs::remove_dir_all(target_dir).await?;
            }
        }

        // Create symlink to store
        #[cfg(unix)]
        {
            tokio::fs::symlink(&stored.store_path, target_dir).await?;
        }

        #[cfg(windows)]
        {
            // On Windows, use directory junction
            std::os::windows::fs::symlink_dir(&stored.store_path, target_dir)?;
        }

        debug!(
            "Linked {} -> {}",
            target_dir.display(),
            stored.store_path.display()
        );

        Ok(())
    }

    /// Hard link files from store to target (for better compatibility).
    pub async fn hard_link_package(&self, integrity: &str, target_dir: &Path) -> Result<()> {
        let stored = self.get_package(integrity).ok_or_else(|| {
            SnpmError::Other(format!(
                "Package with integrity {} not found in store",
                integrity
            ))
        })?;

        // Create target directory
        fs::create_dir_all(target_dir).await?;

        // Hard link all files
        hard_link_dir(&stored.store_path, target_dir).await?;

        debug!(
            "Hard linked {} -> {}",
            stored.store_path.display(),
            target_dir.display()
        );

        Ok(())
    }

    /// Calculate the store path for an integrity hash.
    fn calculate_store_path(&self, integrity: &str, name: &str) -> PathBuf {
        // Extract hash from integrity string (sha512-XXXX -> XXXX)
        let hash = integrity.split('-').last().unwrap_or(integrity);

        // Use first 2 chars as bucket
        let bucket = if hash.len() >= 2 { &hash[..2] } else { "00" };

        self.store_dir
            .join(STORE_VERSION)
            .join("files")
            .join(bucket)
            .join(hash)
            .join("node_modules")
            .join(name.replace('/', "+")) // Handle scoped packages
    }

    /// Load the store index from disk.
    fn load_index(&self) -> Result<()> {
        let files_dir = self.store_dir.join(STORE_VERSION).join("files");

        if !files_dir.exists() {
            return Ok(());
        }

        // Walk through buckets
        for bucket_entry in std::fs::read_dir(&files_dir)? {
            let bucket_entry = bucket_entry?;
            if !bucket_entry.file_type()?.is_dir() {
                continue;
            }

            // Walk through hashes in bucket
            for hash_entry in std::fs::read_dir(bucket_entry.path())? {
                let hash_entry = hash_entry?;
                if !hash_entry.file_type()?.is_dir() {
                    continue;
                }

                let hash = hash_entry.file_name().to_string_lossy().to_string();
                let node_modules = hash_entry.path().join("node_modules");

                if !node_modules.exists() {
                    continue;
                }

                // Find package in node_modules
                for pkg_entry in std::fs::read_dir(&node_modules)? {
                    let pkg_entry = pkg_entry?;
                    let pkg_path = pkg_entry.path();
                    let pkg_name = pkg_entry.file_name().to_string_lossy().replace('+', "/");

                    // Read package.json for version
                    let pkg_json_path = pkg_path.join("package.json");
                    if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
                        if let Ok(pkg_json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let version =
                                pkg_json["version"].as_str().unwrap_or("0.0.0").to_string();

                            let integrity = format!("sha512-{}", hash);
                            let size = calculate_dir_size(&pkg_path).unwrap_or(0);

                            self.index.insert(
                                integrity.clone(),
                                StoredPackage {
                                    name: pkg_name,
                                    version,
                                    integrity,
                                    store_path: pkg_path,
                                    size,
                                },
                            );
                        }
                    }
                }
            }
        }

        debug!("Loaded {} packages from store index", self.index.len());
        Ok(())
    }

    /// Get store statistics.
    pub fn stats(&self) -> StoreStats {
        let mut stats = StoreStats::default();
        let mut packages_by_name: HashMap<String, Vec<u64>> = HashMap::new();

        for entry in self.index.iter() {
            stats.package_count += 1;
            stats.total_size += entry.size;
            packages_by_name
                .entry(entry.name.clone())
                .or_default()
                .push(entry.size);
        }

        stats.unique_packages = packages_by_name.len();

        // Estimate space saved (packages that appear multiple times)
        for (_, sizes) in &packages_by_name {
            if sizes.len() > 1 {
                // Each additional reference saves the package size
                let avg_size = sizes.iter().sum::<u64>() / sizes.len() as u64;
                stats.space_saved += avg_size * (sizes.len() as u64 - 1);
            }
        }

        stats
    }

    /// Prune unused packages from the store.
    pub async fn prune(&self, keep_packages: &[String]) -> Result<PruneResult> {
        let mut result = PruneResult::default();

        let mut to_remove = Vec::new();

        for entry in self.index.iter() {
            if !keep_packages.contains(&entry.integrity) {
                to_remove.push((entry.key().clone(), entry.store_path.clone(), entry.size));
            }
        }

        for (integrity, path, size) in to_remove {
            if path.exists() {
                fs::remove_dir_all(&path).await?;
                result.removed_count += 1;
                result.freed_bytes += size;
            }
            self.index.remove(&integrity);
        }

        info!(
            "Pruned {} packages, freed {} bytes",
            result.removed_count, result.freed_bytes
        );

        Ok(result)
    }

    /// List all packages in the store.
    pub fn list_packages(&self) -> Vec<StoredPackage> {
        self.index.iter().map(|r| r.value().clone()).collect()
    }

    /// Set whether to use hard links (true) or symlinks (false).
    pub fn set_use_hard_links(&mut self, use_hard_links: bool) {
        self.use_hard_links = use_hard_links;
    }
}

/// Result of a prune operation.
#[derive(Debug, Default)]
pub struct PruneResult {
    /// Number of packages removed
    pub removed_count: usize,
    /// Bytes freed
    pub freed_bytes: u64,
}

/// Default store directory.
fn default_store_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".snpm-store")
}

/// Calculate directory size recursively.
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;

    if path.is_file() {
        return Ok(std::fs::metadata(path)?.len());
    }

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry.map_err(|e| SnpmError::Other(e.to_string()))?;
        if entry.file_type().is_file() {
            size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Ok(size)
}

/// Extract a tarball asynchronously.
async fn extract_tarball_async(tarball_path: &Path, dest_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tarball_path = tarball_path.to_path_buf();
    let dest_path = dest_path.to_path_buf();

    // Run extraction in blocking task
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&tarball_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(&dest_path)?;
        Ok::<_, SnpmError>(())
    })
    .await
    .map_err(|e| SnpmError::Other(e.to_string()))??;

    Ok(())
}

/// Hard link a directory recursively.
async fn hard_link_dir(src: &Path, dst: &Path) -> Result<()> {
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();

    tokio::task::spawn_blocking(move || hard_link_dir_sync(&src, &dst))
        .await
        .map_err(|e| SnpmError::Other(e.to_string()))?
}

fn hard_link_dir_sync(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            hard_link_dir_sync(&src_path, &dst_path)?;
        } else {
            // Try hard link first, fall back to copy
            if std::fs::hard_link(&src_path, &dst_path).is_err() {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

/// Virtual store for managing node_modules layout.
///
/// Creates a pnpm-style node_modules structure:
/// ```text
/// node_modules/
/// ├── .pnpm/
/// │   ├── lodash@4.17.21/
/// │   │   └── node_modules/
/// │   │       └── lodash -> <store>
/// │   └── express@4.18.2/
/// │       └── node_modules/
/// │           ├── express -> <store>
/// │           └── body-parser -> ../../body-parser@1.20.2/node_modules/body-parser
/// ├── lodash -> .pnpm/lodash@4.17.21/node_modules/lodash
/// └── express -> .pnpm/express@4.18.2/node_modules/express
/// ```
pub struct VirtualStore {
    /// Root node_modules directory
    node_modules: PathBuf,
    /// Virtual store directory (.pnpm)
    virtual_store: PathBuf,
    /// Package store
    store: PackageStore,
}

impl VirtualStore {
    /// Create a new virtual store.
    pub fn new(node_modules: PathBuf, store: PackageStore) -> Self {
        let virtual_store = node_modules.join(".pnpm");
        Self {
            node_modules,
            virtual_store,
            store,
        }
    }

    /// Initialize the virtual store directories.
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.node_modules).await?;
        fs::create_dir_all(&self.virtual_store).await?;

        // Create lock file to prevent concurrent modifications
        let lock_file = self.virtual_store.join("lock.yaml");
        if !lock_file.exists() {
            fs::write(&lock_file, "lockfileVersion: '6.0'\n").await?;
        }

        Ok(())
    }

    /// Install a package into the virtual store.
    pub async fn install_package(
        &self,
        name: &str,
        version: &str,
        integrity: &str,
        dependencies: &HashMap<String, String>,
    ) -> Result<PathBuf> {
        // Create package directory in virtual store
        let pkg_id = format!("{}@{}", name.replace('/', "+"), version);
        let pkg_virtual_dir = self
            .virtual_store
            .join(&pkg_id)
            .join("node_modules")
            .join(name.replace('/', "+"));

        // Link from store to virtual store
        self.store.link_package(integrity, &pkg_virtual_dir).await?;

        // Create symlinks for dependencies in the package's node_modules
        let pkg_node_modules = self.virtual_store.join(&pkg_id).join("node_modules");

        for (dep_name, dep_version) in dependencies {
            let dep_link = pkg_node_modules.join(dep_name.replace('/', "+"));
            let dep_target = self
                .virtual_store
                .join(format!("{}@{}", dep_name.replace('/', "+"), dep_version))
                .join("node_modules")
                .join(dep_name.replace('/', "+"));

            if !dep_link.exists() && dep_target.exists() {
                // Create relative symlink
                let relative_target =
                    pathdiff::diff_paths(&dep_target, &pkg_node_modules).unwrap_or(dep_target);

                #[cfg(unix)]
                tokio::fs::symlink(&relative_target, &dep_link).await.ok();

                #[cfg(windows)]
                std::os::windows::fs::symlink_dir(&relative_target, &dep_link).ok();
            }
        }

        Ok(pkg_virtual_dir)
    }

    /// Create a top-level symlink in node_modules.
    pub async fn create_top_level_link(&self, name: &str, version: &str) -> Result<()> {
        let pkg_id = format!("{}@{}", name.replace('/', "+"), version);
        let target = self
            .virtual_store
            .join(&pkg_id)
            .join("node_modules")
            .join(name.replace('/', "+"));

        let link = if name.contains('/') {
            // Scoped package: create scope directory
            let parts: Vec<&str> = name.splitn(2, '/').collect();
            let scope_dir = self.node_modules.join(parts[0]);
            fs::create_dir_all(&scope_dir).await?;
            scope_dir.join(parts[1])
        } else {
            self.node_modules.join(name)
        };

        // Remove existing link
        if link.exists() || link.is_symlink() {
            if link.is_dir() && !link.is_symlink() {
                fs::remove_dir_all(&link).await?;
            } else {
                fs::remove_file(&link).await?;
            }
        }

        // Create relative symlink
        let relative_target =
            pathdiff::diff_paths(&target, link.parent().unwrap()).unwrap_or(target);

        #[cfg(unix)]
        tokio::fs::symlink(&relative_target, &link).await?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&relative_target, &link)?;

        debug!(
            "Created top-level link: {} -> {}",
            link.display(),
            relative_target.display()
        );

        Ok(())
    }

    /// Get the path to a package in the virtual store.
    pub fn get_package_path(&self, name: &str, version: &str) -> PathBuf {
        let pkg_id = format!("{}@{}", name.replace('/', "+"), version);
        self.virtual_store
            .join(&pkg_id)
            .join("node_modules")
            .join(name.replace('/', "+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_path_calculation() {
        let store = PackageStore::new(Some(PathBuf::from("/tmp/test-store"))).unwrap();
        let path = store.calculate_store_path("sha512-abc123def456", "lodash");
        assert!(path.to_string_lossy().contains("ab"));
        assert!(path.to_string_lossy().contains("lodash"));
    }
}
