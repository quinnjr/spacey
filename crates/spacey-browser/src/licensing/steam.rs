//! Steam Integration - Steamworks SDK bindings
//!
//! This module provides integration with the Steam platform for:
//! - License verification via app ownership
//! - User authentication
//! - DLC/entitlement checking
//! - Cloud saves (future)
//! - Achievements (future)
//! - Steam Overlay (future)

#[cfg(feature = "steam")]
use steamworks::{
    AppId, Client, ClientManager, PersonaStateFlags,
    SingleClient, SteamId, UserStatsReceived,
};

use super::{Entitlement, LicenseError, LicenseUser};

/// Steam App ID for Spacey Browser
/// This should be set to the actual Steam App ID when registered
pub const STEAM_APP_ID: u32 = 480; // Using Spacewar for testing

/// DLC App IDs for Spacey Browser entitlements
pub mod dlc {
    /// AI Local features DLC
    pub const AI_LOCAL: u32 = 0; // Placeholder
    /// AI Pro features DLC
    pub const AI_PRO: u32 = 0; // Placeholder
    /// Extension marketplace DLC
    pub const EXTENSIONS: u32 = 0; // Placeholder
    /// Priority support DLC
    pub const SUPPORT: u32 = 0; // Placeholder
    /// Developer tools DLC
    pub const DEV_TOOLS: u32 = 0; // Placeholder
}

/// Steam license handler
#[cfg(feature = "steam")]
pub struct SteamLicense {
    client: Client,
    _single: SingleClient,
}

#[cfg(feature = "steam")]
impl SteamLicense {
    /// Initialize the Steam client
    pub fn new() -> Result<Self, LicenseError> {
        // Try to initialize Steam
        let (client, single) = Client::init_app(STEAM_APP_ID)
            .map_err(|e| LicenseError::SteamError(format!("Failed to initialize: {:?}", e)))?;

        log::info!("Steam client initialized for app {}", STEAM_APP_ID);

        Ok(Self {
            client,
            _single: single,
        })
    }

    /// Verify that the user owns the game
    pub fn verify_ownership(&self) -> Result<bool, LicenseError> {
        let apps = self.client.apps();

        // Check if app is subscribed (owned)
        let is_subscribed = apps.is_subscribed();

        // Additional checks
        let is_subscribed_app = apps.is_subscribed_app(AppId(STEAM_APP_ID));
        let is_low_violence = apps.is_low_violence();
        let is_cybercafe = apps.is_cybercafe();

        log::debug!(
            "Steam ownership: subscribed={}, app_subscribed={}, low_violence={}, cybercafe={}",
            is_subscribed, is_subscribed_app, is_low_violence, is_cybercafe
        );

        // Valid if user is subscribed to the app
        Ok(is_subscribed && is_subscribed_app)
    }

    /// Get user information from Steam
    pub fn get_user_info(&self) -> Option<LicenseUser> {
        let user = self.client.user();
        let friends = self.client.friends();

        let steam_id = user.steam_id();
        let name = friends.name();

        // Get avatar URL (Steam Community)
        let avatar_url = Some(format!(
            "https://steamcdn-a.akamaihd.net/steamcommunity/public/images/avatars/{}/{}_full.jpg",
            &steam_id.raw().to_string()[..2],
            steam_id.raw()
        ));

        // Get country from IP (if available through Steam)
        let country = self.client.utils().get_ip_country();

        Some(LicenseUser {
            id: steam_id.raw().to_string(),
            name,
            platform: "steam".to_string(),
            avatar_url,
            country: if country.is_empty() { None } else { Some(country) },
        })
    }

    /// Check if user owns a specific DLC
    pub fn owns_dlc(&self, dlc_id: u32) -> bool {
        if dlc_id == 0 {
            return false; // Invalid/placeholder DLC ID
        }

        let apps = self.client.apps();
        apps.is_dlc_installed(AppId(dlc_id))
    }

    /// Get all owned DLC
    pub fn get_owned_dlc(&self) -> Vec<Entitlement> {
        let mut entitlements = Vec::new();
        let apps = self.client.apps();

        // Check each known DLC
        let dlcs = [
            (dlc::AI_LOCAL, "AI Copilot (Local)"),
            (dlc::AI_PRO, "AI Copilot Pro"),
            (dlc::EXTENSIONS, "Extension Marketplace"),
            (dlc::SUPPORT, "Priority Support"),
            (dlc::DEV_TOOLS, "Developer Tools"),
        ];

        for (dlc_id, name) in dlcs {
            if dlc_id == 0 {
                continue; // Skip placeholders
            }

            let is_installed = apps.is_dlc_installed(AppId(dlc_id));

            entitlements.push(Entitlement {
                id: dlc_id.to_string(),
                name: name.to_string(),
                active: is_installed,
            });
        }

        entitlements
    }

    /// Get the Steam ID of the current user
    pub fn steam_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }

    /// Get the user's persona name (display name)
    pub fn persona_name(&self) -> String {
        self.client.friends().name()
    }

    /// Check if Steam overlay is enabled
    pub fn is_overlay_enabled(&self) -> bool {
        self.client.utils().is_overlay_enabled()
    }

    /// Activate Steam overlay to a specific URL
    pub fn activate_overlay_to_url(&self, url: &str) {
        self.client.friends().activate_game_overlay_to_web_page(url);
    }

    /// Activate Steam overlay to store page
    pub fn activate_overlay_to_store(&self, app_id: Option<u32>) {
        let app = app_id.map(AppId).unwrap_or(AppId(STEAM_APP_ID));
        self.client.friends().activate_game_overlay_to_store(app, steamworks::OverlayToStoreFlag::None);
    }

    /// Get the current game language
    pub fn game_language(&self) -> String {
        self.client.apps().current_game_language()
    }

    /// Get available game languages
    pub fn available_languages(&self) -> String {
        self.client.apps().available_game_languages()
    }

    /// Run Steam callbacks - should be called regularly
    pub fn run_callbacks(&self) {
        self._single.run_callbacks();
    }

    /// Get app build ID
    pub fn build_id(&self) -> i32 {
        self.client.apps().app_build_id()
    }

    /// Get app install directory
    pub fn install_dir(&self) -> Option<String> {
        self.client.apps().app_install_dir(AppId(STEAM_APP_ID))
    }

    /// Check if the app is running in VR mode
    pub fn is_vr_mode(&self) -> bool {
        self.client.utils().is_steam_running_in_vr()
    }

    /// Get Steam UI language
    pub fn ui_language(&self) -> String {
        self.client.utils().get_steam_ui_language()
    }
}

/// Stub implementation when Steam feature is disabled
#[cfg(not(feature = "steam"))]
pub struct SteamLicense;

#[cfg(not(feature = "steam"))]
impl SteamLicense {
    pub fn new() -> Result<Self, LicenseError> {
        Err(LicenseError::SteamError("Steam feature not enabled".to_string()))
    }

    pub fn verify_ownership(&self) -> Result<bool, LicenseError> {
        Ok(false)
    }

    pub fn get_user_info(&self) -> Option<LicenseUser> {
        None
    }

    pub fn owns_dlc(&self, _dlc_id: u32) -> bool {
        false
    }

    pub fn get_owned_dlc(&self) -> Vec<Entitlement> {
        Vec::new()
    }

    pub fn run_callbacks(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_app_id() {
        // Using Spacewar test app ID
        assert_eq!(STEAM_APP_ID, 480);
    }

    #[test]
    #[cfg(not(feature = "steam"))]
    fn test_steam_disabled() {
        let result = SteamLicense::new();
        assert!(result.is_err());
    }
}
