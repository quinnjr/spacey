//! Extension Loader
//!
//! Handles loading extensions from disk and the Firefox AMO marketplace.
//! Supports both XPI (ZIP) files and unpacked extensions.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::extensions::manifest::{ExtensionManifest, ManifestError};
use parking_lot::RwLock;

/// Unique identifier for an extension
pub type ExtensionId = String;

/// An installed extension
#[derive(Debug, Clone)]
pub struct Extension {
    /// Unique identifier
    pub id: ExtensionId,
    /// Parsed manifest
    pub manifest: ExtensionManifest,
    /// Path to extension files
    pub path: PathBuf,
    /// Whether the extension is enabled
    pub enabled: bool,
    /// Whether this is a temporary (developer) install
    pub temporary: bool,
    /// Installation source
    pub source: InstallSource,
}

/// Where the extension was installed from
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// Installed from AMO
    Amo { addon_id: String, version: String },
    /// Loaded from local directory
    Local,
    /// Installed from XPI file
    Xpi { original_path: PathBuf },
    /// Built-in extension
    Builtin,
}

/// Extension loader and manager
pub struct ExtensionLoader {
    /// Directory where extensions are stored
    extensions_dir: PathBuf,
    /// Loaded extensions
    extensions: HashMap<ExtensionId, Extension>,
    /// Extension load order
    load_order: Vec<ExtensionId>,
}

impl ExtensionLoader {
    /// Create a new extension loader
    pub fn new(extensions_dir: PathBuf) -> Self {
        // Create extensions directory if it doesn't exist
        if !extensions_dir.exists() {
            fs::create_dir_all(&extensions_dir).ok();
        }

        Self {
            extensions_dir,
            extensions: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Get the extensions directory
    pub fn extensions_dir(&self) -> &Path {
        &self.extensions_dir
    }

    /// Load all installed extensions
    pub fn load_all(&mut self) -> Result<Vec<ExtensionId>, ExtensionError> {
        let mut loaded = Vec::new();

        // Read the extensions directory
        let entries = fs::read_dir(&self.extensions_dir)
            .map_err(|e| ExtensionError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| ExtensionError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                // Unpacked extension
                match self.load_unpacked(&path) {
                    Ok(id) => loaded.push(id),
                    Err(e) => log::warn!("Failed to load extension from {:?}: {}", path, e),
                }
            } else if path.extension().map(|e| e == "xpi").unwrap_or(false) {
                // XPI file - extract and load
                match self.install_xpi(&path, false) {
                    Ok(id) => loaded.push(id),
                    Err(e) => log::warn!("Failed to load XPI {:?}: {}", path, e),
                }
            }
        }

        Ok(loaded)
    }

    /// Load an unpacked extension from a directory
    pub fn load_unpacked(&mut self, path: &Path) -> Result<ExtensionId, ExtensionError> {
        let manifest_path = path.join("manifest.json");

        if !manifest_path.exists() {
            return Err(ExtensionError::ManifestNotFound);
        }

        let manifest = ExtensionManifest::from_file(&manifest_path)
            .map_err(ExtensionError::ManifestError)?;

        manifest.validate().map_err(ExtensionError::ManifestError)?;

        // Generate or get extension ID
        let id = manifest.get_id().unwrap_or_else(|| {
            // Generate ID from name if not specified
            format!("{}@spacey.local", manifest.name.to_lowercase().replace(' ', "-"))
        });

        let extension = Extension {
            id: id.clone(),
            manifest,
            path: path.to_path_buf(),
            enabled: true,
            temporary: false,
            source: InstallSource::Local,
        };

        self.extensions.insert(id.clone(), extension);
        self.load_order.push(id.clone());

        log::info!("Loaded extension: {} ({})", id, path.display());
        Ok(id)
    }

    /// Load a temporary (developer) extension
    pub fn load_temporary(&mut self, path: &Path) -> Result<ExtensionId, ExtensionError> {
        let id = self.load_unpacked(path)?;

        if let Some(ext) = self.extensions.get_mut(&id) {
            ext.temporary = true;
        }

        Ok(id)
    }

    /// Install an extension from an XPI file
    pub fn install_xpi(&mut self, xpi_path: &Path, keep_xpi: bool) -> Result<ExtensionId, ExtensionError> {
        let xpi_file = fs::File::open(xpi_path)
            .map_err(|e| ExtensionError::IoError(e.to_string()))?;

        let mut archive = zip::ZipArchive::new(xpi_file)
            .map_err(|e| ExtensionError::InvalidXpi(e.to_string()))?;

        // First, read the manifest to get the extension ID
        let manifest_content = {
            let mut manifest_file = archive.by_name("manifest.json")
                .map_err(|_| ExtensionError::ManifestNotFound)?;
            let mut content = String::new();
            manifest_file.read_to_string(&mut content)
                .map_err(|e| ExtensionError::IoError(e.to_string()))?;
            content
        };

        let manifest = ExtensionManifest::from_json(&manifest_content)
            .map_err(ExtensionError::ManifestError)?;

        manifest.validate().map_err(ExtensionError::ManifestError)?;

        let id = manifest.get_id().unwrap_or_else(|| {
            format!("{}@spacey.local", manifest.name.to_lowercase().replace(' ', "-"))
        });

        // Create extension directory
        let ext_dir = self.extensions_dir.join(&id);
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir)
                .map_err(|e| ExtensionError::IoError(e.to_string()))?;
        }
        fs::create_dir_all(&ext_dir)
            .map_err(|e| ExtensionError::IoError(e.to_string()))?;

        // Re-open archive for extraction
        let xpi_file = fs::File::open(xpi_path)
            .map_err(|e| ExtensionError::IoError(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(xpi_file)
            .map_err(|e| ExtensionError::InvalidXpi(e.to_string()))?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| ExtensionError::InvalidXpi(e.to_string()))?;

            let outpath = match file.enclosed_name() {
                Some(path) => ext_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| ExtensionError::IoError(e.to_string()))?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| ExtensionError::IoError(e.to_string()))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| ExtensionError::IoError(e.to_string()))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| ExtensionError::IoError(e.to_string()))?;
            }
        }

        // Create extension entry
        let extension = Extension {
            id: id.clone(),
            manifest,
            path: ext_dir,
            enabled: true,
            temporary: false,
            source: InstallSource::Xpi {
                original_path: xpi_path.to_path_buf(),
            },
        };

        self.extensions.insert(id.clone(), extension);
        self.load_order.push(id.clone());

        log::info!("Installed extension from XPI: {}", id);
        Ok(id)
    }

    /// Uninstall an extension
    pub fn uninstall(&mut self, id: &str) -> Result<(), ExtensionError> {
        let extension = self.extensions.remove(id)
            .ok_or(ExtensionError::NotFound)?;

        self.load_order.retain(|i| i != id);

        // Don't delete temporary extensions' files
        if !extension.temporary {
            if extension.path.exists() {
                fs::remove_dir_all(&extension.path)
                    .map_err(|e| ExtensionError::IoError(e.to_string()))?;
            }
        }

        log::info!("Uninstalled extension: {}", id);
        Ok(())
    }

    /// Enable an extension
    pub fn enable(&mut self, id: &str) -> Result<(), ExtensionError> {
        let extension = self.extensions.get_mut(id)
            .ok_or(ExtensionError::NotFound)?;
        extension.enabled = true;
        Ok(())
    }

    /// Disable an extension
    pub fn disable(&mut self, id: &str) -> Result<(), ExtensionError> {
        let extension = self.extensions.get_mut(id)
            .ok_or(ExtensionError::NotFound)?;
        extension.enabled = false;
        Ok(())
    }

    /// Get an extension by ID
    pub fn get(&self, id: &str) -> Option<&Extension> {
        self.extensions.get(id)
    }

    /// Get all extensions
    pub fn all(&self) -> impl Iterator<Item = &Extension> {
        self.extensions.values()
    }

    /// Get all enabled extensions in load order
    pub fn enabled(&self) -> Vec<&Extension> {
        self.load_order
            .iter()
            .filter_map(|id| self.extensions.get(id))
            .filter(|ext| ext.enabled)
            .collect()
    }

    /// Get extensions that have content scripts matching a URL
    pub fn get_content_scripts_for_url(&self, url: &str) -> Vec<(&Extension, &crate::extensions::manifest::ContentScript)> {
        let mut result = Vec::new();

        for ext in self.enabled() {
            for cs in &ext.manifest.content_scripts {
                if Self::url_matches_patterns(url, &cs.matches, &cs.exclude_matches) {
                    result.push((ext, cs));
                }
            }
        }

        result
    }

    /// Check if a URL matches the given patterns
    fn url_matches_patterns(url: &str, include: &[String], exclude: &[String]) -> bool {
        // Check exclusions first
        for pattern in exclude {
            if Self::url_matches_pattern(url, pattern) {
                return false;
            }
        }

        // Check inclusions
        for pattern in include {
            if Self::url_matches_pattern(url, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a URL matches a single pattern
    fn url_matches_pattern(url: &str, pattern: &str) -> bool {
        // Handle special patterns
        if pattern == "<all_urls>" {
            return url.starts_with("http://") || url.starts_with("https://");
        }

        if pattern == "*://*/*" {
            return url.starts_with("http://") || url.starts_with("https://");
        }

        // Convert pattern to regex-like matching
        // Pattern format: scheme://host/path
        // * matches any character sequence
        // ? matches any single character

        let pattern = pattern
            .replace(".", r"\.")
            .replace("*", ".*")
            .replace("?", ".");

        regex::Regex::new(&format!("^{}$", pattern))
            .map(|re| re.is_match(url))
            .unwrap_or(false)
    }

    /// Read a file from an extension
    pub fn read_extension_file(&self, id: &str, path: &str) -> Result<Vec<u8>, ExtensionError> {
        let extension = self.get(id).ok_or(ExtensionError::NotFound)?;
        let file_path = extension.path.join(path);

        fs::read(&file_path).map_err(|e| ExtensionError::IoError(e.to_string()))
    }
}

/// Errors that can occur during extension operations
#[derive(Debug)]
pub enum ExtensionError {
    IoError(String),
    ManifestNotFound,
    ManifestError(ManifestError),
    InvalidXpi(String),
    NotFound,
    NetworkError(String),
    RuntimeError(String),
}

impl From<crate::extensions::runtime::RuntimeError> for ExtensionError {
    fn from(e: crate::extensions::runtime::RuntimeError) -> Self {
        ExtensionError::RuntimeError(e.to_string())
    }
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::IoError(e) => write!(f, "IO error: {}", e),
            ExtensionError::ManifestNotFound => write!(f, "manifest.json not found"),
            ExtensionError::ManifestError(e) => write!(f, "Manifest error: {}", e),
            ExtensionError::InvalidXpi(e) => write!(f, "Invalid XPI: {}", e),
            ExtensionError::NotFound => write!(f, "Extension not found"),
            ExtensionError::NetworkError(e) => write!(f, "Network error: {}", e),
            ExtensionError::RuntimeError(e) => write!(f, "Runtime error: {}", e),
        }
    }
}

impl std::error::Error for ExtensionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_pattern_matching() {
        assert!(ExtensionLoader::url_matches_pattern(
            "https://example.com/page",
            "<all_urls>"
        ));

        assert!(ExtensionLoader::url_matches_pattern(
            "https://example.com/page",
            "*://*/*"
        ));

        assert!(ExtensionLoader::url_matches_pattern(
            "https://example.com/page",
            "https://example.com/*"
        ));

        assert!(!ExtensionLoader::url_matches_pattern(
            "https://other.com/page",
            "https://example.com/*"
        ));
    }
}
