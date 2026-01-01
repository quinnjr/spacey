//! Windows Store (Microsoft Store) Integration
//!
//! This module provides integration with the Microsoft Store for:
//! - License verification via Windows.Services.Store
//! - In-app purchases
//! - Add-on/DLC management
//! - Subscription handling
//!
//! Requires Windows 10 1607+ and the Windows SDK.

use super::{Entitlement, LicenseError, LicenseUser};

/// Microsoft Store Product ID for Spacey Browser
/// This should be set to the actual Store ID when published
pub const MS_STORE_ID: &str = "9XXXXXXXXXX"; // Placeholder

/// Add-on Store IDs for Spacey Browser entitlements
pub mod addons {
    /// AI Local features add-on
    pub const AI_LOCAL: &str = "spacey.ai.local";
    /// AI Pro features add-on
    pub const AI_PRO: &str = "spacey.ai.pro";
    /// Extension marketplace add-on
    pub const EXTENSIONS: &str = "spacey.extensions";
    /// Priority support subscription
    pub const SUPPORT: &str = "spacey.support";
    /// Developer tools add-on
    pub const DEV_TOOLS: &str = "spacey.devtools";
}

/// License type from Microsoft Store
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MsLicenseType {
    /// Full purchased license
    Full,
    /// Trial license with time limit
    Trial { days_remaining: u32 },
    /// Subscription license
    Subscription { expires_at: u64 },
    /// Developer/sideloaded license
    Developer,
    /// No license
    None,
}

/// Windows Store license handler
#[cfg(all(windows, feature = "windows-store"))]
pub struct WindowsStoreLicense {
    /// Store context for API calls
    context: windows::Services::Store::StoreContext,
    /// Current license info
    license: Option<windows::Services::Store::StoreAppLicense>,
    /// User info
    user_info: Option<LicenseUser>,
}

#[cfg(all(windows, feature = "windows-store"))]
impl WindowsStoreLicense {
    /// Initialize the Windows Store license handler
    pub fn new() -> Result<Self, LicenseError> {
        use windows::Services::Store::StoreContext;
        
        let context = StoreContext::GetDefault()
            .map_err(|e| LicenseError::WindowsStoreError(format!("Failed to get store context: {:?}", e)))?;
        
        log::info!("Windows Store context initialized");
        
        Ok(Self {
            context,
            license: None,
            user_info: None,
        })
    }
    
    /// Verify that the user has a valid license
    pub async fn verify_license(&mut self) -> Result<MsLicenseType, LicenseError> {
        use windows::Foundation::IAsyncOperation;
        
        let license_op = self.context.GetAppLicenseAsync()
            .map_err(|e| LicenseError::WindowsStoreError(format!("Failed to get license: {:?}", e)))?;
        
        let license = license_op.await
            .map_err(|e| LicenseError::WindowsStoreError(format!("License query failed: {:?}", e)))?;
        
        self.license = Some(license.clone());
        
        // Check license status
        if !license.IsActive()
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))? 
        {
            return Ok(MsLicenseType::None);
        }
        
        // Check if trial
        if license.IsTrial()
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))? 
        {
            let trial_info = license.TrialTimeRemaining()
                .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
            
            let days = trial_info.Days as u32;
            return Ok(MsLicenseType::Trial { days_remaining: days });
        }
        
        // Full license
        Ok(MsLicenseType::Full)
    }
    
    /// Get user information
    pub async fn get_user_info(&mut self) -> Option<LicenseUser> {
        use windows::System::User;
        
        // Try to get the current user
        let users = match User::FindAllAsync() {
            Ok(op) => match op.await {
                Ok(users) => users,
                Err(_) => return None,
            },
            Err(_) => return None,
        };
        
        if users.Size().unwrap_or(0) == 0 {
            return None;
        }
        
        let user = match users.GetAt(0) {
            Ok(u) => u,
            Err(_) => return None,
        };
        
        // Get user properties
        let display_name = user.GetPropertyAsync(
            windows::System::KnownUserProperties::DisplayName().ok()?
        ).ok()?.await.ok()?.try_into().ok()?;
        
        let account_name = user.GetPropertyAsync(
            windows::System::KnownUserProperties::AccountName().ok()?
        ).ok()?.await.ok()?.try_into().ok()?;
        
        let user_info = LicenseUser {
            id: account_name,
            name: display_name,
            platform: "windows_store".to_string(),
            avatar_url: None, // Windows doesn't expose avatar URL directly
            country: None,
        };
        
        self.user_info = Some(user_info.clone());
        Some(user_info)
    }
    
    /// Check if user owns a specific add-on
    pub async fn owns_addon(&self, addon_id: &str) -> bool {
        let license = match &self.license {
            Some(l) => l,
            None => return false,
        };
        
        let addons = match license.AddOnLicenses() {
            Ok(a) => a,
            Err(_) => return false,
        };
        
        // Check each add-on license
        for i in 0..addons.Size().unwrap_or(0) {
            if let Ok(addon) = addons.GetAt(i) {
                if let Ok(sku) = addon.SkuStoreId() {
                    if sku.to_string() == addon_id {
                        if let Ok(active) = addon.IsActive() {
                            return active;
                        }
                    }
                }
            }
        }
        
        false
    }
    
    /// Get all owned add-ons
    pub async fn get_owned_addons(&self) -> Vec<Entitlement> {
        let mut entitlements = Vec::new();
        
        let license = match &self.license {
            Some(l) => l,
            None => return entitlements,
        };
        
        let addons = match license.AddOnLicenses() {
            Ok(a) => a,
            Err(_) => return entitlements,
        };
        
        for i in 0..addons.Size().unwrap_or(0) {
            if let Ok(addon) = addons.GetAt(i) {
                let id = addon.SkuStoreId()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                    
                let active = addon.IsActive().unwrap_or(false);
                
                // Map to known entitlements
                let name = match id.as_str() {
                    id if id == addons::AI_LOCAL => "AI Copilot (Local)",
                    id if id == addons::AI_PRO => "AI Copilot Pro",
                    id if id == addons::EXTENSIONS => "Extension Marketplace",
                    id if id == addons::SUPPORT => "Priority Support",
                    id if id == addons::DEV_TOOLS => "Developer Tools",
                    _ => continue,
                };
                
                entitlements.push(Entitlement {
                    id,
                    name: name.to_string(),
                    active,
                });
            }
        }
        
        entitlements
    }
    
    /// Purchase an add-on
    pub async fn purchase_addon(&self, addon_id: &str) -> Result<bool, LicenseError> {
        let result = self.context.RequestPurchaseAsync(&addon_id.into())
            .map_err(|e| LicenseError::WindowsStoreError(format!("Purchase request failed: {:?}", e)))?
            .await
            .map_err(|e| LicenseError::WindowsStoreError(format!("Purchase failed: {:?}", e)))?;
        
        // Check purchase status
        match result.Status() {
            Ok(status) => {
                use windows::Services::Store::StorePurchaseStatus;
                match status {
                    StorePurchaseStatus::Succeeded => Ok(true),
                    StorePurchaseStatus::AlreadyPurchased => Ok(true),
                    StorePurchaseStatus::NotPurchased => Ok(false),
                    _ => Err(LicenseError::WindowsStoreError("Purchase cancelled or failed".to_string())),
                }
            }
            Err(e) => Err(LicenseError::WindowsStoreError(e.to_string())),
        }
    }
    
    /// Open the Store page for the app
    pub async fn open_store_page(&self) -> Result<(), LicenseError> {
        use windows::System::Launcher;
        
        let uri = windows::Foundation::Uri::CreateUri(
            &format!("ms-windows-store://pdp/?ProductId={}", MS_STORE_ID).into()
        ).map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
        
        Launcher::LaunchUriAsync(&uri)
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?
            .await
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<bool, LicenseError> {
        let updates = self.context.GetAppAndOptionalStorePackageUpdatesAsync()
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?
            .await
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
        
        Ok(updates.Size().unwrap_or(0) > 0)
    }
    
    /// Download and install updates
    pub async fn install_updates(&self) -> Result<(), LicenseError> {
        let updates = self.context.GetAppAndOptionalStorePackageUpdatesAsync()
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?
            .await
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
        
        if updates.Size().unwrap_or(0) == 0 {
            return Ok(());
        }
        
        self.context.RequestDownloadAndInstallStorePackageUpdatesAsync(&updates)
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?
            .await
            .map_err(|e| LicenseError::WindowsStoreError(e.to_string()))?;
        
        Ok(())
    }
}

/// Stub implementation when Windows Store feature is disabled
#[cfg(not(all(windows, feature = "windows-store")))]
pub struct WindowsStoreLicense;

#[cfg(not(all(windows, feature = "windows-store")))]
impl WindowsStoreLicense {
    pub fn new() -> Result<Self, LicenseError> {
        Err(LicenseError::WindowsStoreError("Windows Store feature not enabled or not on Windows".to_string()))
    }
    
    pub async fn verify_license(&mut self) -> Result<MsLicenseType, LicenseError> {
        Ok(MsLicenseType::None)
    }
    
    pub async fn get_user_info(&mut self) -> Option<LicenseUser> {
        None
    }
    
    pub async fn owns_addon(&self, _addon_id: &str) -> bool {
        false
    }
    
    pub async fn get_owned_addons(&self) -> Vec<Entitlement> {
        Vec::new()
    }
    
    pub async fn purchase_addon(&self, _addon_id: &str) -> Result<bool, LicenseError> {
        Err(LicenseError::WindowsStoreError("Windows Store not available".to_string()))
    }
    
    pub async fn open_store_page(&self) -> Result<(), LicenseError> {
        Err(LicenseError::WindowsStoreError("Windows Store not available".to_string()))
    }
    
    pub async fn check_for_updates(&self) -> Result<bool, LicenseError> {
        Ok(false)
    }
    
    pub async fn install_updates(&self) -> Result<(), LicenseError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ms_store_id() {
        assert!(!MS_STORE_ID.is_empty());
    }
    
    #[test]
    fn test_addon_ids() {
        assert!(!addons::AI_LOCAL.is_empty());
        assert!(!addons::AI_PRO.is_empty());
    }
    
    #[test]
    #[cfg(not(all(windows, feature = "windows-store")))]
    fn test_windows_store_disabled() {
        let result = WindowsStoreLicense::new();
        assert!(result.is_err());
    }
}
