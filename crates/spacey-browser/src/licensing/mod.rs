//! Licensing Module - Steam and other platform integrations
//!
//! This module provides license verification and authentication through
//! various platforms, primarily Steam.

pub mod steam;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// License status for the application
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    /// License verified and valid
    Valid,
    /// License is being verified
    Verifying,
    /// No license found (trial mode)
    Trial { days_remaining: u32 },
    /// License verification failed
    Invalid { reason: String },
    /// Running in offline mode with cached license
    OfflineValid { expires_at: u64 },
    /// Steam not available (standalone mode)
    Standalone,
}

impl Default for LicenseStatus {
    fn default() -> Self {
        LicenseStatus::Verifying
    }
}

/// User information from the licensing platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseUser {
    /// Platform user ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Platform (e.g., "steam", "standalone")
    pub platform: String,
    /// Avatar URL if available
    pub avatar_url: Option<String>,
    /// User's country code
    pub country: Option<String>,
}

/// Entitlements/DLC the user owns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    /// Entitlement ID (e.g., DLC app ID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this entitlement is active
    pub active: bool,
}

/// Known entitlements for Spacey Browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpaceyEntitlement {
    /// Base browser license
    Browser,
    /// AI features (local model)
    AiLocal,
    /// Pro AI features (cloud providers)
    AiPro,
    /// Extension marketplace access
    Extensions,
    /// Priority support
    Support,
    /// Developer tools
    DevTools,
}

impl SpaceyEntitlement {
    /// Get the Steam DLC app ID for this entitlement
    pub fn steam_dlc_id(&self) -> Option<u32> {
        match self {
            // These would be actual Steam DLC app IDs when registered
            SpaceyEntitlement::Browser => None, // Base game, not DLC
            SpaceyEntitlement::AiLocal => Some(0), // Placeholder
            SpaceyEntitlement::AiPro => Some(0),   // Placeholder
            SpaceyEntitlement::Extensions => Some(0), // Placeholder
            SpaceyEntitlement::Support => Some(0),    // Placeholder
            SpaceyEntitlement::DevTools => Some(0),   // Placeholder
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            SpaceyEntitlement::Browser => "Spacey Browser",
            SpaceyEntitlement::AiLocal => "AI Copilot (Local)",
            SpaceyEntitlement::AiPro => "AI Copilot Pro",
            SpaceyEntitlement::Extensions => "Extension Marketplace",
            SpaceyEntitlement::Support => "Priority Support",
            SpaceyEntitlement::DevTools => "Developer Tools",
        }
    }
}

/// License manager - handles all licensing operations
pub struct LicenseManager {
    /// Current license status
    status: Arc<RwLock<LicenseStatus>>,
    /// Current user info
    user: Arc<RwLock<Option<LicenseUser>>>,
    /// Owned entitlements
    entitlements: Arc<RwLock<Vec<Entitlement>>>,
    /// Steam client (if available)
    #[cfg(feature = "steam")]
    steam: Option<steam::SteamLicense>,
    /// Offline license cache path
    cache_path: std::path::PathBuf,
}

impl LicenseManager {
    /// Create a new license manager
    pub fn new() -> Self {
        let cache_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("spacey")
            .join("license.cache");

        Self {
            status: Arc::new(RwLock::new(LicenseStatus::Verifying)),
            user: Arc::new(RwLock::new(None)),
            entitlements: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "steam")]
            steam: None,
            cache_path,
        }
    }

    /// Initialize the license manager
    ///
    /// This will:
    /// 1. Try to initialize Steam if available
    /// 2. Verify the license
    /// 3. Load user information
    /// 4. Check entitlements
    #[cfg(feature = "steam")]
    pub fn initialize(&mut self) -> Result<(), LicenseError> {
        log::info!("Initializing license manager...");

        // Try Steam first
        match steam::SteamLicense::new() {
            Ok(steam) => {
                log::info!("Steam client initialized");

                // Verify ownership
                if steam.verify_ownership()? {
                    *self.status.write() = LicenseStatus::Valid;

                    // Load user info
                    if let Some(user) = steam.get_user_info() {
                        log::info!("Steam user: {} ({})", user.name, user.id);
                        *self.user.write() = Some(user);
                    }

                    // Check entitlements
                    let owned = steam.get_owned_dlc();
                    *self.entitlements.write() = owned;

                    // Cache license for offline use
                    self.cache_license()?;

                    self.steam = Some(steam);
                } else {
                    *self.status.write() = LicenseStatus::Invalid {
                        reason: "Steam ownership verification failed".to_string(),
                    };
                }
            }
            Err(e) => {
                log::warn!("Steam not available: {}", e);

                // Try offline cache
                if let Ok(cached) = self.load_cached_license() {
                    *self.status.write() = cached;
                } else {
                    // Fall back to standalone/trial mode
                    *self.status.write() = LicenseStatus::Standalone;
                }
            }
        }

        Ok(())
    }

    /// Initialize without Steam (standalone mode)
    #[cfg(not(feature = "steam"))]
    pub fn initialize(&mut self) -> Result<(), LicenseError> {
        log::info!("Initializing license manager (standalone mode)...");

        // Try offline cache
        if let Ok(cached) = self.load_cached_license() {
            *self.status.write() = cached;
        } else {
            // Standalone mode - all features unlocked
            *self.status.write() = LicenseStatus::Standalone;
        }

        Ok(())
    }

    /// Get the current license status
    pub fn status(&self) -> LicenseStatus {
        self.status.read().clone()
    }

    /// Check if a specific entitlement is owned
    pub fn has_entitlement(&self, entitlement: SpaceyEntitlement) -> bool {
        // In standalone mode, all entitlements are available
        if matches!(*self.status.read(), LicenseStatus::Standalone) {
            return true;
        }

        // Check if license is valid
        if !matches!(*self.status.read(), LicenseStatus::Valid | LicenseStatus::OfflineValid { .. }) {
            return false;
        }

        // Base browser is always available with valid license
        if matches!(entitlement, SpaceyEntitlement::Browser) {
            return true;
        }

        // Check DLC entitlements
        #[cfg(feature = "steam")]
        if let Some(dlc_id) = entitlement.steam_dlc_id() {
            if let Some(ref steam) = self.steam {
                return steam.owns_dlc(dlc_id);
            }
        }

        // Check cached entitlements
        let entitlements = self.entitlements.read();
        entitlements.iter().any(|e| e.name == entitlement.name() && e.active)
    }

    /// Get the current user
    pub fn user(&self) -> Option<LicenseUser> {
        self.user.read().clone()
    }

    /// Get all owned entitlements
    pub fn owned_entitlements(&self) -> Vec<Entitlement> {
        self.entitlements.read().clone()
    }

    /// Check if we're running through Steam
    #[cfg(feature = "steam")]
    pub fn is_steam(&self) -> bool {
        self.steam.is_some()
    }

    #[cfg(not(feature = "steam"))]
    pub fn is_steam(&self) -> bool {
        false
    }

    /// Run Steam callbacks (should be called periodically)
    #[cfg(feature = "steam")]
    pub fn run_callbacks(&mut self) {
        if let Some(ref steam) = self.steam {
            steam.run_callbacks();
        }
    }

    #[cfg(not(feature = "steam"))]
    pub fn run_callbacks(&mut self) {
        // No-op without Steam
    }

    /// Cache license for offline use
    fn cache_license(&self) -> Result<(), LicenseError> {
        let status = self.status.read().clone();
        let user = self.user.read().clone();
        let entitlements = self.entitlements.read().clone();

        let cache = LicenseCache {
            status,
            user,
            entitlements,
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 7 * 24 * 60 * 60, // 7 days
        };

        // Create directory if needed
        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        }

        let json = serde_json::to_string_pretty(&cache)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;

        std::fs::write(&self.cache_path, json)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;

        log::info!("License cached for offline use");
        Ok(())
    }

    /// Load cached license
    fn load_cached_license(&self) -> Result<LicenseStatus, LicenseError> {
        let data = std::fs::read_to_string(&self.cache_path)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;

        let cache: LicenseCache = serde_json::from_str(&data)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;

        // Check if cache is still valid
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > cache.expires_at {
            return Err(LicenseError::CacheExpired);
        }

        // Restore user and entitlements
        *self.user.write() = cache.user;
        *self.entitlements.write() = cache.entitlements;

        log::info!("Loaded cached license (expires in {} days)",
            (cache.expires_at - now) / (24 * 60 * 60));

        Ok(LicenseStatus::OfflineValid { expires_at: cache.expires_at })
    }
}

impl Default for LicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached license data for offline use
#[derive(Debug, Serialize, Deserialize)]
struct LicenseCache {
    status: LicenseStatus,
    user: Option<LicenseUser>,
    entitlements: Vec<Entitlement>,
    cached_at: u64,
    expires_at: u64,
}

/// License-related errors
#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("Steam initialization failed: {0}")]
    SteamError(String),

    #[error("License verification failed: {0}")]
    VerificationFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Cache expired")]
    CacheExpired,

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_status_default() {
        let status = LicenseStatus::default();
        assert!(matches!(status, LicenseStatus::Verifying));
    }

    #[test]
    fn test_entitlement_names() {
        assert_eq!(SpaceyEntitlement::Browser.name(), "Spacey Browser");
        assert_eq!(SpaceyEntitlement::AiPro.name(), "AI Copilot Pro");
    }

    #[test]
    fn test_license_manager_creation() {
        let manager = LicenseManager::new();
        assert!(matches!(manager.status(), LicenseStatus::Verifying));
    }
}
