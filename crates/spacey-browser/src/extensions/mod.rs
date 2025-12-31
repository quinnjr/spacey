//! Firefox-compatible Extension System
//!
//! Provides full WebExtensions API support with emphasis on:
//! - **Full webRequest blocking** (unlike Chrome's crippled MV3)
//! - **AMO integration** (Firefox marketplace)
//! - **Manifest V2 and V3** support
//! - **Content blockers** (uBlock Origin, etc.)
//!
//! # Philosophy
//!
//! Unlike Chrome, which has systematically weakened extension capabilities
//! to protect its advertising business, Spacey Browser provides FULL
//! extension support:
//!
//! 1. **webRequest with blocking**: Extensions can inspect and modify
//!    requests BEFORE they're sent. This is essential for ad blockers.
//!
//! 2. **Manifest V2 forever**: We don't deprecate V2 because V3's
//!    declarativeNetRequest is insufficient for advanced blocking.
//!
//! 3. **AMO integration**: Direct access to Firefox's extension marketplace
//!    with all the privacy-focused extensions that work best with full APIs.

pub mod manifest;
pub mod loader;
pub mod amo;
pub mod runtime;
pub mod apis;

pub use manifest::{ExtensionManifest, ManifestVersion, ManifestError};
pub use loader::{Extension, ExtensionLoader, ExtensionError, ExtensionId, InstallSource};
pub use amo::{AmoClient, AmoError, AddonDetail, AddonSummary, SearchResults};
pub use runtime::{ExtensionRuntime, RuntimeError};
pub use apis::{
    ExtensionStorage, StorageArea, StorageError,
    WebRequestApi, RequestDetails, BlockingResponse, RequestAction, ResourceType,
};

use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

/// Extension manager - main interface for the extension system
pub struct ExtensionManager {
    /// Extension loader
    loader: Arc<RwLock<ExtensionLoader>>,
    /// Extension runtime
    runtime: Arc<ExtensionRuntime>,
    /// AMO client
    amo: AmoClient,
    /// Data directory
    data_dir: PathBuf,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new(data_dir: PathBuf) -> Self {
        let extensions_dir = data_dir.join("extensions");
        let loader = Arc::new(RwLock::new(ExtensionLoader::new(extensions_dir)));
        let runtime = Arc::new(ExtensionRuntime::new(data_dir.clone()));
        let amo = AmoClient::new();

        Self {
            loader,
            runtime,
            amo,
            data_dir,
        }
    }

    /// Initialize - load all installed extensions
    pub fn init(&self) -> Result<Vec<ExtensionId>, ExtensionError> {
        let ids = self.loader.write().load_all()?;

        // Initialize each extension's runtime
        let loader = self.loader.read();
        for ext in loader.enabled() {
            if let Err(e) = self.runtime.init_extension(ext) {
                log::error!("Failed to initialize extension {}: {}", ext.id, e);
            }
        }

        log::info!("Loaded {} extensions", ids.len());
        Ok(ids)
    }

    /// Install an extension from AMO
    pub fn install_from_amo(&self, slug: &str) -> Result<ExtensionId, ExtensionError> {
        log::info!("Installing extension from AMO: {}", slug);

        // Get addon details
        let addon = self.amo.get_addon(slug)
            .map_err(|e| ExtensionError::NetworkError(e.to_string()))?;

        // Download XPI
        let xpi_path = self.data_dir.join("downloads").join(format!("{}.xpi", slug));
        
        if let Some(parent) = xpi_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        self.amo.download_xpi(&addon, &xpi_path)
            .map_err(|e| ExtensionError::NetworkError(e.to_string()))?;

        // Install the XPI
        let id = self.loader.write().install_xpi(&xpi_path, false)?;

        // Initialize the extension
        if let Some(ext) = self.loader.read().get(&id) {
            self.runtime.init_extension(ext)?;
        }

        // Clean up XPI
        std::fs::remove_file(&xpi_path).ok();

        Ok(id)
    }

    /// Install from local XPI file
    pub fn install_xpi(&self, path: &std::path::Path) -> Result<ExtensionId, ExtensionError> {
        let id = self.loader.write().install_xpi(path, false)?;

        if let Some(ext) = self.loader.read().get(&id) {
            self.runtime.init_extension(ext)?;
        }

        Ok(id)
    }

    /// Load a temporary extension (for development)
    pub fn load_temporary(&self, path: &std::path::Path) -> Result<ExtensionId, ExtensionError> {
        let id = self.loader.write().load_temporary(path)?;

        if let Some(ext) = self.loader.read().get(&id) {
            self.runtime.init_extension(ext)?;
        }

        Ok(id)
    }

    /// Uninstall an extension
    pub fn uninstall(&self, id: &str) -> Result<(), ExtensionError> {
        self.runtime.unload_extension(id);
        self.loader.write().uninstall(id)
    }

    /// Enable an extension
    pub fn enable(&self, id: &str) -> Result<(), ExtensionError> {
        self.loader.write().enable(id)?;

        if let Some(ext) = self.loader.read().get(id) {
            self.runtime.init_extension(ext)?;
        }

        Ok(())
    }

    /// Disable an extension
    pub fn disable(&self, id: &str) -> Result<(), ExtensionError> {
        self.runtime.unload_extension(id);
        self.loader.write().disable(id)
    }

    /// Search AMO for extensions
    pub fn search_amo(&self, query: &str) -> Result<SearchResults, AmoError> {
        self.amo.search(query, 1, 25)
    }

    /// Get featured extensions from AMO
    pub fn get_featured(&self) -> Result<SearchResults, AmoError> {
        self.amo.get_featured(25)
    }

    /// Get recommended content blockers
    pub fn get_recommended_blockers(&self) -> Result<Vec<AddonSummary>, AmoError> {
        self.amo.get_recommended_blockers()
    }

    /// Get all installed extensions
    pub fn list(&self) -> Vec<Extension> {
        self.loader.read().all().cloned().collect()
    }

    /// Get an extension by ID
    pub fn get(&self, id: &str) -> Option<Extension> {
        self.loader.read().get(id).cloned()
    }

    /// Get the extension runtime
    pub fn runtime(&self) -> Arc<ExtensionRuntime> {
        Arc::clone(&self.runtime)
    }

    /// Get the loader
    pub fn loader(&self) -> Arc<RwLock<ExtensionLoader>> {
        Arc::clone(&self.loader)
    }

    /// Process a network request through all extensions
    pub fn process_request(&self, details: &RequestDetails) -> RequestAction {
        let responses = self.runtime.webrequest().process_before_request(details);
        self.runtime.webrequest().apply_blocking_responses(&responses)
    }

    /// Get extensions with content scripts for a URL
    pub fn get_content_scripts_for_url(&self, url: &str) -> Vec<(Extension, manifest::ContentScript)> {
        let loader = self.loader.read();
        let content_scripts = loader.get_content_scripts_for_url(url);
        
        content_scripts
            .into_iter()
            .map(|(ext, cs)| (ext.clone(), cs.clone()))
            .collect()
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new(dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extension_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ExtensionManager::new(temp_dir.path().to_path_buf());
        
        assert!(manager.list().is_empty());
    }
}
