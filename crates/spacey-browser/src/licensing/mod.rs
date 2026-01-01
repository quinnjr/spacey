//! Licensing Module - Steam and other platform integrations
//!
//! This module provides license verification and authentication through
//! various platforms, primarily Steam. Also includes a 14-day free trial
//! for users without a license.

pub mod steam;
pub mod windows;
pub mod apple;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use parking_lot::RwLock;

/// Duration of the free trial in days
pub const TRIAL_DURATION_DAYS: u32 = 14;

/// Seconds in a day
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// License status for the application
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    /// License verified and valid
    Valid,
    /// License is being verified
    Verifying,
    /// No license found - running in trial mode
    Trial { 
        /// Days remaining in trial
        days_remaining: u32,
        /// Whether this is the first run
        first_run: bool,
    },
    /// Trial has expired
    TrialExpired,
    /// License verification failed
    Invalid { reason: String },
    /// Running in offline mode with cached license
    OfflineValid { expires_at: u64 },
    /// Running in standalone/developer mode (all features unlocked)
    Standalone,
}

impl LicenseStatus {
    /// Check if the user has access to features
    pub fn has_access(&self) -> bool {
        matches!(
            self,
            LicenseStatus::Valid
                | LicenseStatus::Trial { days_remaining: 1..=u32::MAX, .. }
                | LicenseStatus::OfflineValid { .. }
                | LicenseStatus::Standalone
        )
    }
    
    /// Check if this is a trial
    pub fn is_trial(&self) -> bool {
        matches!(self, LicenseStatus::Trial { .. })
    }
    
    /// Check if trial has expired
    pub fn is_expired(&self) -> bool {
        matches!(self, LicenseStatus::TrialExpired | LicenseStatus::Trial { days_remaining: 0, .. })
    }
    
    /// Get days remaining (for trial)
    pub fn days_remaining(&self) -> Option<u32> {
        match self {
            LicenseStatus::Trial { days_remaining, .. } => Some(*days_remaining),
            _ => None,
        }
    }
}

impl Default for LicenseStatus {
    fn default() -> Self {
        LicenseStatus::Verifying
    }
}

/// Trial information stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialInfo {
    /// Timestamp when trial started (Unix epoch seconds)
    pub started_at: u64,
    /// Timestamp when trial expires
    pub expires_at: u64,
    /// Machine identifier for tamper detection
    pub machine_id: String,
    /// Number of times the app has been launched during trial
    pub launch_count: u32,
    /// Version that started the trial
    pub version: String,
}

impl TrialInfo {
    /// Create a new trial starting now
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            started_at: now,
            expires_at: now + (TRIAL_DURATION_DAYS as u64 * SECONDS_PER_DAY),
            machine_id: Self::generate_machine_id(),
            launch_count: 1,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Calculate days remaining in trial
    pub fn days_remaining(&self) -> u32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if now >= self.expires_at {
            0
        } else {
            ((self.expires_at - now) / SECONDS_PER_DAY) as u32 + 1
        }
    }
    
    /// Check if trial has expired
    pub fn is_expired(&self) -> bool {
        self.days_remaining() == 0
    }
    
    /// Validate the trial hasn't been tampered with
    pub fn is_valid(&self) -> bool {
        // Check machine ID matches
        if self.machine_id != Self::generate_machine_id() {
            return false;
        }
        
        // Check timestamps are reasonable
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Trial can't start in the future
        if self.started_at > now {
            return false;
        }
        
        // Expiry should be start + trial duration
        let expected_expiry = self.started_at + (TRIAL_DURATION_DAYS as u64 * SECONDS_PER_DAY);
        if self.expires_at != expected_expiry {
            return false;
        }
        
        true
    }
    
    /// Generate a machine-specific identifier
    fn generate_machine_id() -> String {
        // Use a combination of system properties to create a stable ID
        let mut id_parts = Vec::new();
        
        // Try to get hostname
        if let Ok(hostname) = hostname::get() {
            id_parts.push(hostname.to_string_lossy().to_string());
        }
        
        // Add username
        if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            id_parts.push(user);
        }
        
        // Add home directory
        if let Some(home) = dirs::home_dir() {
            id_parts.push(format!("{:x}", hash_path(&home)));
        }
        
        // Create a simple hash of all parts
        let combined = id_parts.join("|");
        format!("{:016x}", simple_hash(&combined))
    }
}

impl Default for TrialInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple non-cryptographic hash for identifiers
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

/// Hash a path for machine ID
fn hash_path(path: &std::path::Path) -> u64 {
    simple_hash(&path.to_string_lossy())
}

/// User information from the licensing platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseUser {
    /// Platform user ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Platform (e.g., "steam", "trial", "standalone")
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
    
    /// Check if this entitlement is available during trial
    pub fn available_in_trial(&self) -> bool {
        match self {
            // All basic features available in trial
            SpaceyEntitlement::Browser => true,
            SpaceyEntitlement::AiLocal => true,
            SpaceyEntitlement::Extensions => true,
            SpaceyEntitlement::DevTools => true,
            // Pro features require purchase
            SpaceyEntitlement::AiPro => false,
            SpaceyEntitlement::Support => false,
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

/// The platform the app is running on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    /// Steam (any OS)
    Steam,
    /// Microsoft Store (Windows)
    WindowsStore,
    /// Apple App Store (macOS/iOS)
    AppleStore,
    /// Standalone (direct download, trial mode)
    Standalone,
}

impl Platform {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Steam => "Steam",
            Platform::WindowsStore => "Microsoft Store",
            Platform::AppleStore => "App Store",
            Platform::Standalone => "Standalone",
        }
    }
    
    /// Get store URL
    pub fn store_url(&self) -> Option<&'static str> {
        match self {
            Platform::Steam => Some("https://store.steampowered.com/app/0"), // Placeholder
            Platform::WindowsStore => Some("ms-windows-store://pdp/?ProductId=9XXXXXXXXXX"),
            Platform::AppleStore => Some("macappstore://apps.apple.com/app/spacey-browser"),
            Platform::Standalone => Some("https://spacey.pegasusheavy.dev"),
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
    /// Trial information
    trial_info: Arc<RwLock<Option<TrialInfo>>>,
    /// Detected platform
    platform: Platform,
    /// Steam client (if available)
    #[cfg(feature = "steam")]
    steam: Option<steam::SteamLicense>,
    /// Windows Store client (if available)
    #[cfg(all(windows, feature = "windows-store"))]
    windows_store: Option<windows::WindowsStoreLicense>,
    /// Apple Store client (if available)
    #[cfg(all(target_os = "macos", feature = "apple-store"))]
    apple_store: Option<apple::AppStoreLicense>,
    /// Data directory for license/trial storage
    data_dir: PathBuf,
    /// Offline license cache path
    cache_path: PathBuf,
    /// Trial info path
    trial_path: PathBuf,
}

impl LicenseManager {
    /// Create a new license manager
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("spacey");
        
        let cache_path = data_dir.join("license.cache");
        let trial_path = data_dir.join(".trial");
        
        Self {
            status: Arc::new(RwLock::new(LicenseStatus::Verifying)),
            user: Arc::new(RwLock::new(None)),
            entitlements: Arc::new(RwLock::new(Vec::new())),
            trial_info: Arc::new(RwLock::new(None)),
            platform: Platform::Standalone,
            #[cfg(feature = "steam")]
            steam: None,
            #[cfg(all(windows, feature = "windows-store"))]
            windows_store: None,
            #[cfg(all(target_os = "macos", feature = "apple-store"))]
            apple_store: None,
            data_dir,
            cache_path,
            trial_path,
        }
    }
    
    /// Detect the platform we're running on
    fn detect_platform() -> Platform {
        // Check Steam first (cross-platform)
        #[cfg(feature = "steam")]
        {
            if std::env::var("SteamAppId").is_ok() || std::env::var("SteamClientLaunch").is_ok() {
                return Platform::Steam;
            }
        }
        
        // Check Windows Store
        #[cfg(all(windows, feature = "windows-store"))]
        {
            if Self::is_windows_store_package() {
                return Platform::WindowsStore;
            }
        }
        
        // Check Apple Store
        #[cfg(all(target_os = "macos", feature = "apple-store"))]
        {
            if Self::is_apple_store_app() {
                return Platform::AppleStore;
            }
        }
        
        Platform::Standalone
    }
    
    #[cfg(all(windows, feature = "windows-store"))]
    fn is_windows_store_package() -> bool {
        std::env::var("MSIX_PACKAGE_NAME").is_ok() 
            || std::path::Path::new("C:\\Program Files\\WindowsApps").exists()
                && std::env::current_exe()
                    .map(|p| p.to_string_lossy().contains("WindowsApps"))
                    .unwrap_or(false)
    }
    
    #[cfg(not(all(windows, feature = "windows-store")))]
    fn is_windows_store_package() -> bool {
        false
    }
    
    #[cfg(all(target_os = "macos", feature = "apple-store"))]
    fn is_apple_store_app() -> bool {
        // Check for MAS (Mac App Store) receipt
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .map(|app_dir| app_dir.join("../_MASReceipt/receipt").exists())
            .unwrap_or(false)
    }
    
    #[cfg(not(all(target_os = "macos", feature = "apple-store")))]
    fn is_apple_store_app() -> bool {
        false
    }
    
    /// Get the current platform
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Initialize the license manager
    ///
    /// This will:
    /// 1. Detect the platform
    /// 2. Initialize the appropriate store client
    /// 3. Verify the license
    /// 4. Load user information
    /// 5. Check entitlements
    /// 6. Fall back to trial mode if no license found
    pub fn initialize(&mut self) -> Result<(), LicenseError> {
        log::info!("Initializing license manager...");
        
        // Detect platform
        self.platform = Self::detect_platform();
        log::info!("Detected platform: {:?}", self.platform);
        
        match self.platform {
            Platform::Steam => self.initialize_steam(),
            Platform::WindowsStore => self.initialize_windows_store(),
            Platform::AppleStore => self.initialize_apple_store(),
            Platform::Standalone => self.initialize_standalone(),
        }
    }
    
    /// Initialize Steam
    #[cfg(feature = "steam")]
    fn initialize_steam(&mut self) -> Result<(), LicenseError> {
        match steam::SteamLicense::new() {
            Ok(steam) => {
                log::info!("Steam client initialized");

                if steam.verify_ownership()? {
                    *self.status.write() = LicenseStatus::Valid;

                    if let Some(user) = steam.get_user_info() {
                        log::info!("Steam user: {} ({})", user.name, user.id);
                        *self.user.write() = Some(user);
                    }

                    let owned = steam.get_owned_dlc();
                    *self.entitlements.write() = owned;
                    self.cache_license()?;
                    self.steam = Some(steam);
                    return Ok(());
                } else {
                    log::info!("Steam ownership not verified, checking trial...");
                }
            }
            Err(e) => {
                log::warn!("Steam not available: {}", e);
            }
        }

        self.fallback_to_cache_or_trial()
    }
    
    #[cfg(not(feature = "steam"))]
    fn initialize_steam(&mut self) -> Result<(), LicenseError> {
        self.initialize_standalone()
    }
    
    /// Initialize Windows Store
    #[cfg(all(windows, feature = "windows-store"))]
    fn initialize_windows_store(&mut self) -> Result<(), LicenseError> {
        match windows::WindowsStoreLicense::new() {
            Ok(store) => {
                log::info!("Windows Store client initialized");
                
                // Note: Full implementation would use async/await
                // For now, we'll cache the store and verify lazily
                self.windows_store = Some(store);
                *self.status.write() = LicenseStatus::Valid;
                
                *self.user.write() = Some(LicenseUser {
                    id: "windows_store".to_string(),
                    name: whoami::username(),
                    platform: "windows_store".to_string(),
                    avatar_url: None,
                    country: None,
                });
                
                self.cache_license()?;
                Ok(())
            }
            Err(e) => {
                log::warn!("Windows Store not available: {}", e);
                self.fallback_to_cache_or_trial()
            }
        }
    }
    
    #[cfg(not(all(windows, feature = "windows-store")))]
    fn initialize_windows_store(&mut self) -> Result<(), LicenseError> {
        self.initialize_standalone()
    }
    
    /// Initialize Apple Store
    #[cfg(all(target_os = "macos", feature = "apple-store"))]
    fn initialize_apple_store(&mut self) -> Result<(), LicenseError> {
        match apple::AppStoreLicense::new() {
            Ok(store) => {
                log::info!("Apple Store client initialized");
                
                self.apple_store = Some(store);
                // App Store apps are inherently licensed
                *self.status.write() = LicenseStatus::Valid;
                
                *self.user.write() = Some(LicenseUser {
                    id: "apple_store".to_string(),
                    name: whoami::username(),
                    platform: "apple_store".to_string(),
                    avatar_url: None,
                    country: None,
                });
                
                self.cache_license()?;
                Ok(())
            }
            Err(e) => {
                log::warn!("Apple Store not available: {}", e);
                self.fallback_to_cache_or_trial()
            }
        }
    }
    
    #[cfg(not(all(target_os = "macos", feature = "apple-store")))]
    fn initialize_apple_store(&mut self) -> Result<(), LicenseError> {
        self.initialize_standalone()
    }
    
    /// Initialize standalone mode (with trial)
    fn initialize_standalone(&mut self) -> Result<(), LicenseError> {
        log::info!("Initializing in standalone mode...");

        // Try offline cache first
        if let Ok(cached) = self.load_cached_license() {
            *self.status.write() = cached;
            return Ok(());
        }

        // Fall back to trial mode
        self.initialize_trial()
    }
    
    /// Fallback to cache or trial mode
    fn fallback_to_cache_or_trial(&mut self) -> Result<(), LicenseError> {
        if let Ok(cached) = self.load_cached_license() {
            *self.status.write() = cached;
            Ok(())
        } else {
            self.initialize_trial()
        }
    }
    
    /// Initialize or continue trial mode
    fn initialize_trial(&mut self) -> Result<(), LicenseError> {
        // Create data directory if needed
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        // Check for existing trial
        if let Ok(trial) = self.load_trial_info() {
            if trial.is_valid() {
                let days = trial.days_remaining();
                
                if days == 0 {
                    log::info!("Trial has expired");
                    *self.status.write() = LicenseStatus::TrialExpired;
                } else {
                    log::info!("Continuing trial: {} days remaining", days);
                    *self.status.write() = LicenseStatus::Trial {
                        days_remaining: days,
                        first_run: false,
                    };
                    
                    // Update launch count
                    let mut updated_trial = trial;
                    updated_trial.launch_count += 1;
                    self.save_trial_info(&updated_trial)?;
                    *self.trial_info.write() = Some(updated_trial);
                }
                
                // Set trial user
                *self.user.write() = Some(LicenseUser {
                    id: "trial".to_string(),
                    name: "Trial User".to_string(),
                    platform: "trial".to_string(),
                    avatar_url: None,
                    country: None,
                });
                
                return Ok(());
            } else {
                log::warn!("Trial info invalid, may have been tampered with");
            }
        }
        
        // Start new trial
        log::info!("Starting {} day free trial", TRIAL_DURATION_DAYS);
        let trial = TrialInfo::new();
        self.save_trial_info(&trial)?;
        
        *self.trial_info.write() = Some(trial);
        *self.status.write() = LicenseStatus::Trial {
            days_remaining: TRIAL_DURATION_DAYS,
            first_run: true,
        };
        
        // Set trial user
        *self.user.write() = Some(LicenseUser {
            id: "trial".to_string(),
            name: "Trial User".to_string(),
            platform: "trial".to_string(),
            avatar_url: None,
            country: None,
        });
        
        Ok(())
    }
    
    /// Load trial info from disk
    fn load_trial_info(&self) -> Result<TrialInfo, LicenseError> {
        let data = std::fs::read_to_string(&self.trial_path)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        // Decode from base64 for slight obfuscation
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            data.trim(),
        ).map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        let json = String::from_utf8(decoded)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        serde_json::from_str(&json)
            .map_err(|e| LicenseError::CacheError(e.to_string()))
    }
    
    /// Save trial info to disk
    fn save_trial_info(&self, trial: &TrialInfo) -> Result<(), LicenseError> {
        let json = serde_json::to_string(trial)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        // Encode to base64 for slight obfuscation
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            json.as_bytes(),
        );
        
        std::fs::write(&self.trial_path, encoded)
            .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        
        Ok(())
    }

    /// Get the current license status
    pub fn status(&self) -> LicenseStatus {
        self.status.read().clone()
    }
    
    /// Get trial info if in trial mode
    pub fn trial_info(&self) -> Option<TrialInfo> {
        self.trial_info.read().clone()
    }
    
    /// Check if currently in trial mode
    pub fn is_trial(&self) -> bool {
        self.status.read().is_trial()
    }
    
    /// Check if trial has expired
    pub fn is_trial_expired(&self) -> bool {
        self.status.read().is_expired()
    }
    
    /// Get days remaining in trial (None if not in trial)
    pub fn trial_days_remaining(&self) -> Option<u32> {
        self.status.read().days_remaining()
    }

    /// Check if a specific entitlement is owned
    pub fn has_entitlement(&self, entitlement: SpaceyEntitlement) -> bool {
        let status = self.status.read().clone();
        
        // In standalone mode, all entitlements are available
        if matches!(status, LicenseStatus::Standalone) {
            return true;
        }
        
        // Check trial mode
        if let LicenseStatus::Trial { days_remaining, .. } = status {
            if days_remaining > 0 {
                return entitlement.available_in_trial();
            }
            return false;
        }
        
        // Trial expired - no access
        if matches!(status, LicenseStatus::TrialExpired) {
            return false;
        }

        // Check if license is valid
        if !matches!(status, LicenseStatus::Valid | LicenseStatus::OfflineValid { .. }) {
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
                .as_secs() + 7 * SECONDS_PER_DAY, // 7 days
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
            (cache.expires_at - now) / SECONDS_PER_DAY);

        Ok(LicenseStatus::OfflineValid { expires_at: cache.expires_at })
    }
    
    /// Activate a license key (placeholder for future implementation)
    pub fn activate_license(&mut self, _license_key: &str) -> Result<(), LicenseError> {
        // TODO: Implement license key activation
        // This would validate the key with a server and upgrade from trial
        Err(LicenseError::VerificationFailed("License key activation not yet implemented".to_string()))
    }
    
    /// Reset trial (for testing only, would be removed in production)
    #[cfg(debug_assertions)]
    pub fn reset_trial(&mut self) -> Result<(), LicenseError> {
        if self.trial_path.exists() {
            std::fs::remove_file(&self.trial_path)
                .map_err(|e| LicenseError::CacheError(e.to_string()))?;
        }
        *self.trial_info.write() = None;
        *self.status.write() = LicenseStatus::Verifying;
        self.initialize_trial()
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
    
    #[error("Windows Store error: {0}")]
    WindowsStoreError(String),
    
    #[error("Apple App Store error: {0}")]
    AppleStoreError(String),

    #[error("License verification failed: {0}")]
    VerificationFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Cache expired")]
    CacheExpired,

    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Trial expired")]
    TrialExpired,
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
    fn test_license_status_has_access() {
        assert!(LicenseStatus::Valid.has_access());
        assert!(LicenseStatus::Standalone.has_access());
        assert!(LicenseStatus::Trial { days_remaining: 7, first_run: false }.has_access());
        assert!(!LicenseStatus::Trial { days_remaining: 0, first_run: false }.has_access());
        assert!(!LicenseStatus::TrialExpired.has_access());
    }
    
    #[test]
    fn test_trial_info_creation() {
        let trial = TrialInfo::new();
        assert_eq!(trial.days_remaining(), TRIAL_DURATION_DAYS);
        assert!(!trial.is_expired());
        assert_eq!(trial.launch_count, 1);
    }
    
    #[test]
    fn test_trial_info_expiry() {
        let mut trial = TrialInfo::new();
        // Set expiry to the past
        trial.expires_at = trial.started_at - 1;
        assert!(trial.is_expired());
        assert_eq!(trial.days_remaining(), 0);
    }

    #[test]
    fn test_entitlement_names() {
        assert_eq!(SpaceyEntitlement::Browser.name(), "Spacey Browser");
        assert_eq!(SpaceyEntitlement::AiPro.name(), "AI Copilot Pro");
    }
    
    #[test]
    fn test_entitlement_trial_availability() {
        assert!(SpaceyEntitlement::Browser.available_in_trial());
        assert!(SpaceyEntitlement::AiLocal.available_in_trial());
        assert!(!SpaceyEntitlement::AiPro.available_in_trial());
        assert!(!SpaceyEntitlement::Support.available_in_trial());
    }

    #[test]
    fn test_license_manager_creation() {
        let manager = LicenseManager::new();
        assert!(matches!(manager.status(), LicenseStatus::Verifying));
    }
    
    #[test]
    fn test_simple_hash() {
        let hash1 = simple_hash("test");
        let hash2 = simple_hash("test");
        let hash3 = simple_hash("different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
