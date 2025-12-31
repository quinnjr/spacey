//! Extension Manifest Parser
//!
//! Supports both Manifest V2 and V3 with full compatibility.
//! Unlike Chrome, we maintain FULL support for Manifest V2 features,
//! particularly the blocking webRequest API that extensions like
//! uBlock Origin depend on.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Manifest version - we support both V2 and V3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestVersion {
    V2 = 2,
    V3 = 3,
}

impl Default for ManifestVersion {
    fn default() -> Self {
        ManifestVersion::V2
    }
}

/// The extension manifest (manifest.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Manifest version (2 or 3)
    pub manifest_version: u8,
    
    /// Extension name
    pub name: String,
    
    /// Extension version
    pub version: String,
    
    /// Description
    #[serde(default)]
    pub description: String,
    
    /// Author
    #[serde(default)]
    pub author: Option<String>,
    
    /// Homepage URL
    #[serde(default)]
    pub homepage_url: Option<String>,
    
    /// Icons
    #[serde(default)]
    pub icons: HashMap<String, String>,
    
    /// Permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    
    /// Optional permissions
    #[serde(default)]
    pub optional_permissions: Vec<String>,
    
    /// Host permissions (MV3) or included in permissions (MV2)
    #[serde(default)]
    pub host_permissions: Vec<String>,
    
    /// Background scripts/service worker
    #[serde(default)]
    pub background: Option<BackgroundConfig>,
    
    /// Content scripts
    #[serde(default)]
    pub content_scripts: Vec<ContentScript>,
    
    /// Browser action (MV2) / Action (MV3)
    #[serde(default)]
    pub browser_action: Option<BrowserAction>,
    
    /// Action (MV3 equivalent of browser_action)
    #[serde(default)]
    pub action: Option<BrowserAction>,
    
    /// Page action
    #[serde(default)]
    pub page_action: Option<PageAction>,
    
    /// Options page
    #[serde(default)]
    pub options_page: Option<String>,
    
    /// Options UI
    #[serde(default)]
    pub options_ui: Option<OptionsUI>,
    
    /// Web accessible resources
    #[serde(default)]
    pub web_accessible_resources: Vec<WebAccessibleResource>,
    
    /// Content Security Policy
    #[serde(default)]
    pub content_security_policy: Option<ContentSecurityPolicy>,
    
    /// Commands (keyboard shortcuts)
    #[serde(default)]
    pub commands: HashMap<String, Command>,
    
    /// Browser-specific settings
    #[serde(default)]
    pub browser_specific_settings: Option<BrowserSpecificSettings>,
    
    /// Applications (legacy Firefox)
    #[serde(default)]
    pub applications: Option<BrowserSpecificSettings>,
}

/// Background script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BackgroundConfig {
    /// MV2 style with scripts array
    Scripts {
        scripts: Vec<String>,
        #[serde(default)]
        persistent: bool,
    },
    /// MV3 style with service worker
    ServiceWorker {
        service_worker: String,
        #[serde(default = "default_module_type")]
        r#type: String,
    },
    /// MV2 page style
    Page {
        page: String,
    },
}

fn default_module_type() -> String {
    "classic".to_string()
}

/// Content script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    /// URL patterns to match
    pub matches: Vec<String>,
    
    /// URLs to exclude
    #[serde(default)]
    pub exclude_matches: Vec<String>,
    
    /// Glob patterns to include
    #[serde(default)]
    pub include_globs: Vec<String>,
    
    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    
    /// JavaScript files to inject
    #[serde(default)]
    pub js: Vec<String>,
    
    /// CSS files to inject
    #[serde(default)]
    pub css: Vec<String>,
    
    /// When to inject (document_start, document_end, document_idle)
    #[serde(default = "default_run_at")]
    pub run_at: String,
    
    /// Whether to inject into all frames
    #[serde(default)]
    pub all_frames: bool,
    
    /// Match about:blank
    #[serde(default)]
    pub match_about_blank: bool,
}

fn default_run_at() -> String {
    "document_idle".to_string()
}

/// Browser action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAction {
    /// Default icon
    #[serde(default)]
    pub default_icon: Option<IconConfig>,
    
    /// Default title
    #[serde(default)]
    pub default_title: Option<String>,
    
    /// Default popup
    #[serde(default)]
    pub default_popup: Option<String>,
    
    /// Browser style (Firefox)
    #[serde(default)]
    pub browser_style: Option<bool>,
    
    /// Default area (Firefox)
    #[serde(default)]
    pub default_area: Option<String>,
}

/// Icon configuration - can be string or object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IconConfig {
    Single(String),
    Multiple(HashMap<String, String>),
}

/// Page action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAction {
    #[serde(default)]
    pub default_icon: Option<IconConfig>,
    #[serde(default)]
    pub default_title: Option<String>,
    #[serde(default)]
    pub default_popup: Option<String>,
}

/// Options UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsUI {
    pub page: String,
    #[serde(default)]
    pub browser_style: bool,
    #[serde(default)]
    pub open_in_tab: bool,
}

/// Web accessible resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebAccessibleResource {
    /// MV2 style - just a list of paths
    V2(String),
    /// MV3 style - object with resources and matches
    V3 {
        resources: Vec<String>,
        matches: Vec<String>,
        #[serde(default)]
        extension_ids: Vec<String>,
    },
}

/// Content Security Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentSecurityPolicy {
    /// MV2 style - single string
    V2(String),
    /// MV3 style - object
    V3 {
        extension_pages: Option<String>,
        sandbox: Option<String>,
    },
}

/// Command (keyboard shortcut) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    #[serde(default)]
    pub suggested_key: Option<SuggestedKey>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Suggested keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SuggestedKey {
    Simple(String),
    Platform {
        default: Option<String>,
        mac: Option<String>,
        linux: Option<String>,
        windows: Option<String>,
    },
}

/// Browser-specific settings (Firefox)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSpecificSettings {
    #[serde(default)]
    pub gecko: Option<GeckoSettings>,
}

/// Gecko-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeckoSettings {
    /// Extension ID
    pub id: Option<String>,
    /// Minimum Firefox version
    pub strict_min_version: Option<String>,
    /// Maximum Firefox version
    pub strict_max_version: Option<String>,
    /// Update URL
    pub update_url: Option<String>,
}

impl ExtensionManifest {
    /// Parse a manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(json).map_err(ManifestError::ParseError)
    }

    /// Parse a manifest from a file
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::IoError(e.to_string()))?;
        Self::from_json(&contents)
    }

    /// Get the manifest version enum
    pub fn get_version(&self) -> ManifestVersion {
        match self.manifest_version {
            3 => ManifestVersion::V3,
            _ => ManifestVersion::V2,
        }
    }

    /// Check if extension has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
            || self.host_permissions.iter().any(|p| p == permission)
    }

    /// Check if extension needs webRequest blocking (like uBlock Origin)
    pub fn needs_blocking_webrequest(&self) -> bool {
        self.has_permission("webRequest") && self.has_permission("webRequestBlocking")
    }

    /// Get the extension ID (from browser_specific_settings or generate one)
    pub fn get_id(&self) -> Option<String> {
        self.browser_specific_settings
            .as_ref()
            .and_then(|b| b.gecko.as_ref())
            .and_then(|g| g.id.clone())
            .or_else(|| {
                self.applications
                    .as_ref()
                    .and_then(|a| a.gecko.as_ref())
                    .and_then(|g| g.id.clone())
            })
    }

    /// Get the background scripts
    pub fn get_background_scripts(&self) -> Vec<String> {
        match &self.background {
            Some(BackgroundConfig::Scripts { scripts, .. }) => scripts.clone(),
            Some(BackgroundConfig::ServiceWorker { service_worker, .. }) => {
                vec![service_worker.clone()]
            }
            Some(BackgroundConfig::Page { page }) => vec![page.clone()],
            None => vec![],
        }
    }

    /// Check if this is a persistent background page (MV2)
    pub fn is_persistent_background(&self) -> bool {
        match &self.background {
            Some(BackgroundConfig::Scripts { persistent, .. }) => *persistent,
            Some(BackgroundConfig::Page { .. }) => true,
            _ => false,
        }
    }

    /// Get the browser action (handles both MV2 browser_action and MV3 action)
    pub fn get_browser_action(&self) -> Option<&BrowserAction> {
        self.browser_action.as_ref().or(self.action.as_ref())
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::ValidationError("name is required".to_string()));
        }
        if self.version.is_empty() {
            return Err(ManifestError::ValidationError("version is required".to_string()));
        }
        if self.manifest_version != 2 && self.manifest_version != 3 {
            return Err(ManifestError::ValidationError(
                "manifest_version must be 2 or 3".to_string(),
            ));
        }
        Ok(())
    }
}

/// Errors that can occur when parsing a manifest
#[derive(Debug)]
pub enum ManifestError {
    IoError(String),
    ParseError(serde_json::Error),
    ValidationError(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::IoError(e) => write!(f, "IO error: {}", e),
            ManifestError::ParseError(e) => write!(f, "Parse error: {}", e),
            ManifestError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mv2_manifest() {
        let json = r#"{
            "manifest_version": 2,
            "name": "Test Extension",
            "version": "1.0.0",
            "description": "A test extension",
            "permissions": ["webRequest", "webRequestBlocking", "<all_urls>"],
            "background": {
                "scripts": ["background.js"],
                "persistent": true
            },
            "content_scripts": [{
                "matches": ["<all_urls>"],
                "js": ["content.js"],
                "run_at": "document_start"
            }]
        }"#;

        let manifest = ExtensionManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "Test Extension");
        assert_eq!(manifest.get_version(), ManifestVersion::V2);
        assert!(manifest.needs_blocking_webrequest());
    }

    #[test]
    fn test_parse_mv3_manifest() {
        let json = r#"{
            "manifest_version": 3,
            "name": "Test Extension MV3",
            "version": "1.0.0",
            "permissions": ["storage"],
            "host_permissions": ["<all_urls>"],
            "background": {
                "service_worker": "background.js"
            },
            "action": {
                "default_popup": "popup.html"
            }
        }"#;

        let manifest = ExtensionManifest::from_json(json).unwrap();
        assert_eq!(manifest.get_version(), ManifestVersion::V3);
        assert!(!manifest.needs_blocking_webrequest());
    }

    #[test]
    fn test_ublock_style_manifest() {
        // uBlock Origin style manifest with full blocking support
        let json = r#"{
            "manifest_version": 2,
            "name": "uBlock Origin",
            "version": "1.50.0",
            "permissions": [
                "dns",
                "storage",
                "unlimitedStorage",
                "tabs",
                "webNavigation",
                "webRequest",
                "webRequestBlocking",
                "<all_urls>"
            ],
            "background": {
                "scripts": ["js/background.js"],
                "persistent": true
            },
            "browser_specific_settings": {
                "gecko": {
                    "id": "uBlock0@raymondhill.net",
                    "strict_min_version": "78.0"
                }
            }
        }"#;

        let manifest = ExtensionManifest::from_json(json).unwrap();
        assert!(manifest.needs_blocking_webrequest());
        assert!(manifest.is_persistent_background());
        assert_eq!(manifest.get_id(), Some("uBlock0@raymondhill.net".to_string()));
    }
}
