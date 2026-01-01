//! Spacey Shield - Built-in Privacy & Ad Protection
//!
//! A lightweight, built-in protection layer that complements extensions like uBlock Origin.
//!
//! ## Design Philosophy
//!
//! - **Complementary, not competing**: We focus on domain-level blocking and fingerprint
//!   protection while uBlock Origin handles complex cosmetic rules and advanced filtering.
//!
//! - **Minimal by default**: A curated list of the worst offenders, not 300K rules.
//!   This keeps the browser fast while providing baseline protection.
//!
//! - **Different layers**: We block at the network layer (domains), extensions block
//!   at the content layer (elements, scripts). Both work together.
//!
//! ## Features
//!
//! 1. **Domain Blocking**: Block known ad/tracker domains before requests are made
//! 2. **Fingerprint Protection**: Randomize/block fingerprinting APIs
//! 3. **Tracker Isolation**: Prevent cross-site tracking cookies
//! 4. **Upgrade to HTTPS**: Auto-upgrade insecure requests where possible

pub mod blocklist;
pub mod disconnect;
pub mod fingerprint;
pub mod tracker;

use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::extensions::apis::webrequest::{RequestDetails, ResourceType};

pub use blocklist::{BlockList, BlockReason};
pub use disconnect::{DisconnectList, DisconnectCategory, TrackerService};
pub use fingerprint::FingerprintProtection;
pub use tracker::TrackerIsolation;

/// Shield protection levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShieldLevel {
    /// No protection (for troubleshooting)
    Off,
    /// Basic protection - block worst offenders only
    #[default]
    Standard,
    /// Aggressive - may break some sites
    Strict,
}

/// Statistics about blocked content
#[derive(Debug, Clone, Default)]
pub struct ShieldStats {
    /// Total requests blocked
    pub requests_blocked: u64,
    /// Trackers blocked
    pub trackers_blocked: u64,
    /// Ads blocked
    pub ads_blocked: u64,
    /// Social widgets blocked
    pub social_blocked: u64,
    /// Fingerprint attempts blocked
    pub fingerprints_blocked: u64,
    /// Cryptominers blocked
    pub cryptominers_blocked: u64,
    /// Requests upgraded to HTTPS
    pub https_upgrades: u64,
    /// Disconnect list matches
    pub disconnect_blocked: u64,
}

/// The main Spacey Shield protection system
pub struct SpaceyShield {
    /// Protection level
    level: RwLock<ShieldLevel>,
    /// Domain blocklist (Spacey curated)
    blocklist: BlockList,
    /// Disconnect tracking protection list
    disconnect: DisconnectList,
    /// Fingerprint protection
    fingerprint: FingerprintProtection,
    /// Tracker isolation
    tracker_isolation: TrackerIsolation,
    /// Per-site exceptions
    exceptions: RwLock<HashSet<String>>,
    /// Statistics
    stats: RwLock<ShieldStats>,
    /// Whether Disconnect list is enabled
    disconnect_enabled: RwLock<bool>,
}

impl SpaceyShield {
    /// Create a new Spacey Shield instance
    pub fn new() -> Self {
        Self {
            level: RwLock::new(ShieldLevel::Standard),
            blocklist: BlockList::new(),
            disconnect: DisconnectList::new(),
            fingerprint: FingerprintProtection::new(),
            tracker_isolation: TrackerIsolation::new(),
            exceptions: RwLock::new(HashSet::new()),
            stats: RwLock::new(ShieldStats::default()),
            disconnect_enabled: RwLock::new(true), // Enabled by default
        }
    }
    
    /// Enable or disable the Disconnect list
    pub fn set_disconnect_enabled(&self, enabled: bool) {
        *self.disconnect_enabled.write() = enabled;
    }
    
    /// Check if Disconnect list is enabled
    pub fn disconnect_enabled(&self) -> bool {
        *self.disconnect_enabled.read()
    }
    
    /// Get the Disconnect list for inspection
    pub fn disconnect_list(&self) -> &DisconnectList {
        &self.disconnect
    }

    /// Set the protection level
    pub fn set_level(&self, level: ShieldLevel) {
        *self.level.write() = level;
    }

    /// Get the current protection level
    pub fn level(&self) -> ShieldLevel {
        *self.level.read()
    }

    /// Add a site exception (disable blocking for a domain)
    pub fn add_exception(&self, domain: &str) {
        self.exceptions.write().insert(domain.to_lowercase());
    }

    /// Remove a site exception
    pub fn remove_exception(&self, domain: &str) {
        self.exceptions.write().remove(&domain.to_lowercase());
    }

    /// Check if a site has an exception
    pub fn has_exception(&self, domain: &str) -> bool {
        self.exceptions.read().contains(&domain.to_lowercase())
    }

    /// Get current statistics
    pub fn stats(&self) -> ShieldStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = ShieldStats::default();
    }

    /// Check if a request should be blocked
    /// Returns Some(reason) if blocked, None if allowed
    pub fn should_block(&self, details: &RequestDetails) -> Option<BlockReason> {
        let level = *self.level.read();

        if level == ShieldLevel::Off {
            return None;
        }

        // Check site exceptions
        if let Some(ref doc_url) = details.document_url {
            if let Some(domain) = extract_domain(doc_url) {
                if self.has_exception(&domain) {
                    return None;
                }
            }
        }

        // Check domain blocklist (Spacey curated list)
        if let Some(domain) = extract_domain(&details.url) {
            if let Some(reason) = self.blocklist.check(&domain, level) {
                let mut stats = self.stats.write();
                stats.requests_blocked += 1;
                match reason {
                    BlockReason::Advertising => stats.ads_blocked += 1,
                    BlockReason::Tracker => stats.trackers_blocked += 1,
                    BlockReason::Cryptominer => stats.cryptominers_blocked += 1,
                    BlockReason::Fingerprinting => stats.fingerprints_blocked += 1,
                    _ => {}
                }
                return Some(reason);
            }
            
            // Check Disconnect list if enabled
            if *self.disconnect_enabled.read() {
                if let Some(category) = self.disconnect.check(&domain, level) {
                    let mut stats = self.stats.write();
                    stats.requests_blocked += 1;
                    stats.disconnect_blocked += 1;
                    
                    // Map Disconnect category to BlockReason
                    let reason = match category {
                        DisconnectCategory::Advertising => {
                            stats.ads_blocked += 1;
                            BlockReason::Advertising
                        }
                        DisconnectCategory::Analytics => {
                            stats.trackers_blocked += 1;
                            BlockReason::Tracker
                        }
                        DisconnectCategory::Social => {
                            stats.social_blocked += 1;
                            BlockReason::Tracker // Social tracking
                        }
                        DisconnectCategory::Fingerprinting => {
                            stats.fingerprints_blocked += 1;
                            BlockReason::Fingerprinting
                        }
                        DisconnectCategory::Cryptomining => {
                            stats.cryptominers_blocked += 1;
                            BlockReason::Cryptominer
                        }
                        DisconnectCategory::Content => {
                            stats.trackers_blocked += 1;
                            BlockReason::Tracker
                        }
                        DisconnectCategory::Disconnect => {
                            stats.trackers_blocked += 1;
                            BlockReason::Tracker
                        }
                    };
                    return Some(reason);
                }
            }
        }

        // Check for tracking pixels (1x1 images)
        if details.resource_type == ResourceType::Image {
            if self.is_tracking_pixel(&details.url) {
                let mut stats = self.stats.write();
                stats.requests_blocked += 1;
                stats.trackers_blocked += 1;
                return Some(BlockReason::TrackingPixel);
            }
        }

        None
    }

    /// Check if URL should be upgraded to HTTPS
    pub fn should_upgrade_https(&self, url: &str) -> Option<String> {
        if *self.level.read() == ShieldLevel::Off {
            return None;
        }

        if url.starts_with("http://") {
            // Don't upgrade localhost or local IPs
            if url.contains("://localhost")
                || url.contains("://127.0.0.1")
                || url.contains("://192.168.")
                || url.contains("://10.")
            {
                return None;
            }

            let https_url = url.replacen("http://", "https://", 1);
            self.stats.write().https_upgrades += 1;
            return Some(https_url);
        }

        None
    }

    /// Get JavaScript to inject for fingerprint protection
    pub fn get_fingerprint_protection_script(&self) -> Option<String> {
        if *self.level.read() == ShieldLevel::Off {
            return None;
        }

        Some(self.fingerprint.get_protection_script(*self.level.read()))
    }

    /// Check if this looks like a tracking pixel
    fn is_tracking_pixel(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();

        // Common tracking pixel patterns
        url_lower.contains("/pixel")
            || url_lower.contains("/track")
            || url_lower.contains("/beacon")
            || url_lower.contains("1x1")
            || url_lower.contains("spacer.gif")
            || url_lower.contains("blank.gif")
            || url_lower.contains("/p.gif")
            || url_lower.contains("/t.gif")
            || url_lower.ends_with("/p")
            || url_lower.ends_with("/t")
    }

    /// Get blocked domain count
    pub fn blocked_domain_count(&self) -> usize {
        self.blocklist.domain_count()
    }
}

impl Default for SpaceyShield {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract domain from a URL
fn extract_domain(url: &str) -> Option<String> {
    // Remove protocol
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Get domain part (before first /)
    let domain = without_protocol
        .split('/')
        .next()?
        .split(':')
        .next()?  // Remove port
        .to_lowercase();

    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://ads.example.com/tracker.js"),
            Some("ads.example.com".to_string())
        );
        assert_eq!(
            extract_domain("http://example.com:8080/page"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_shield_exceptions() {
        let shield = SpaceyShield::new();

        assert!(!shield.has_exception("example.com"));
        shield.add_exception("example.com");
        assert!(shield.has_exception("example.com"));
        shield.remove_exception("example.com");
        assert!(!shield.has_exception("example.com"));
    }

    #[test]
    fn test_https_upgrade() {
        let shield = SpaceyShield::new();

        assert_eq!(
            shield.should_upgrade_https("http://example.com/page"),
            Some("https://example.com/page".to_string())
        );

        // Don't upgrade localhost
        assert_eq!(shield.should_upgrade_https("http://localhost:3000"), None);

        // Already HTTPS
        assert_eq!(shield.should_upgrade_https("https://example.com"), None);
    }
}
