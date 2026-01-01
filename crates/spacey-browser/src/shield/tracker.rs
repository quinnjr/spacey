//! Tracker Isolation
//!
//! Prevents cross-site tracking by isolating storage and cookies per-site.
//! This complements uBlock Origin's cosmetic filtering with network-level isolation.

use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;

/// Known tracking parameters that can be stripped from URLs
const TRACKING_PARAMS: &[&str] = &[
    // Google Analytics
    "utm_source",
    "utm_medium", 
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "utm_source_platform",
    "utm_creative_format",
    "utm_marketing_tactic",
    
    // Facebook
    "fbclid",
    "fb_action_ids",
    "fb_action_types",
    "fb_source",
    "fb_ref",
    
    // Microsoft
    "msclkid",
    
    // Google Ads
    "gclid",
    "gclsrc",
    "dclid",
    
    // Adobe
    "s_kwcid",
    
    // HubSpot
    "hsa_acc",
    "hsa_ad",
    "hsa_cam",
    "hsa_grp",
    "hsa_kw",
    "hsa_mt",
    "hsa_net",
    "hsa_src",
    "hsa_tgt",
    "hsa_ver",
    
    // Mailchimp
    "mc_cid",
    "mc_eid",
    
    // Twitter
    "twclid",
    
    // Yahoo
    "yclid",
    
    // Generic tracking
    "_ga",
    "_gl",
    "_hsenc",
    "_openstat",
    "mkt_tok",
    "ref",
    "zanpid",
    "igshid",
    "wickedid",
    "oly_enc_id",
    "oly_anon_id",
    "__hsfp",
    "__hssc",
    "__hstc",
    "__s",
    "vero_id",
    "trk_contact",
    "trk_msg",
    "trk_module",
    "trk_sid",
];

/// Tracker isolation for preventing cross-site tracking
pub struct TrackerIsolation {
    /// Sites that have been encountered (for partitioning)
    known_sites: RwLock<HashSet<String>>,
    /// Third-party domain to first-party mappings
    third_party_map: RwLock<HashMap<String, HashSet<String>>>,
}

impl TrackerIsolation {
    pub fn new() -> Self {
        Self {
            known_sites: RwLock::new(HashSet::new()),
            third_party_map: RwLock::new(HashMap::new()),
        }
    }

    /// Record a visit to a site
    pub fn record_site_visit(&self, domain: &str) {
        self.known_sites.write().insert(domain.to_lowercase());
    }

    /// Record a third-party request
    pub fn record_third_party(&self, first_party: &str, third_party: &str) {
        let mut map = self.third_party_map.write();
        map.entry(third_party.to_lowercase())
            .or_insert_with(HashSet::new)
            .insert(first_party.to_lowercase());
    }

    /// Check if a domain appears on multiple first-party sites (likely tracker)
    pub fn is_cross_site_tracker(&self, domain: &str) -> bool {
        let map = self.third_party_map.read();
        if let Some(sites) = map.get(&domain.to_lowercase()) {
            sites.len() > 2 // Appears on more than 2 different sites
        } else {
            false
        }
    }

    /// Strip tracking parameters from a URL
    pub fn strip_tracking_params(&self, url: &str) -> String {
        // Parse URL
        if let Some(question_mark) = url.find('?') {
            let base = &url[..question_mark];
            let query = &url[question_mark + 1..];
            
            // Parse and filter query params
            let filtered: Vec<&str> = query
                .split('&')
                .filter(|param| {
                    let key = param.split('=').next().unwrap_or("");
                    !TRACKING_PARAMS.contains(&key)
                })
                .collect();

            if filtered.is_empty() {
                base.to_string()
            } else {
                format!("{}?{}", base, filtered.join("&"))
            }
        } else {
            url.to_string()
        }
    }

    /// Get JavaScript for tracker isolation
    pub fn get_isolation_script(&self) -> String {
        r#"
(function() {
    'use strict';
    
    // ===== Referrer Policy =====
    // Limit referrer information leaked to third parties
    const meta = document.createElement('meta');
    meta.name = 'referrer';
    meta.content = 'strict-origin-when-cross-origin';
    document.head.appendChild(meta);
    
    // ===== Storage Partitioning =====
    // Note: Full storage partitioning requires browser-level support
    // This provides some JavaScript-level protection
    
    const FIRST_PARTY_DOMAIN = window.location.hostname;
    
    // Wrap localStorage access to partition by domain
    const originalLocalStorage = window.localStorage;
    const partitionedLocalStorage = new Proxy(originalLocalStorage, {
        get: function(target, prop) {
            if (prop === 'getItem') {
                return function(key) {
                    return target.getItem(FIRST_PARTY_DOMAIN + ':' + key);
                };
            }
            if (prop === 'setItem') {
                return function(key, value) {
                    return target.setItem(FIRST_PARTY_DOMAIN + ':' + key, value);
                };
            }
            if (prop === 'removeItem') {
                return function(key) {
                    return target.removeItem(FIRST_PARTY_DOMAIN + ':' + key);
                };
            }
            return target[prop];
        }
    });
    
    // Only apply to third-party iframes
    if (window.top !== window.self) {
        try {
            Object.defineProperty(window, 'localStorage', {
                value: partitionedLocalStorage,
                writable: false
            });
        } catch (e) {
            // May fail in strict contexts
        }
    }
    
    // ===== Link Decoration Removal =====
    // Remove tracking params from clicked links
    document.addEventListener('click', function(e) {
        const link = e.target.closest('a');
        if (link && link.href) {
            const cleanUrl = stripTrackingParams(link.href);
            if (cleanUrl !== link.href) {
                link.href = cleanUrl;
            }
        }
    }, true);
    
    function stripTrackingParams(url) {
        try {
            const u = new URL(url);
            const trackingParams = [
                'utm_source', 'utm_medium', 'utm_campaign', 'utm_term', 'utm_content',
                'fbclid', 'gclid', 'msclkid', 'dclid', 'twclid', 'yclid',
                '_ga', '_gl', 'mc_cid', 'mc_eid', 'igshid'
            ];
            trackingParams.forEach(param => u.searchParams.delete(param));
            return u.toString();
        } catch (e) {
            return url;
        }
    }
    
    console.log('[Spacey Shield] Tracker isolation active');
})();
"#.to_string()
    }
}

impl Default for TrackerIsolation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_tracking_params() {
        let isolation = TrackerIsolation::new();
        
        // Should strip UTM params
        assert_eq!(
            isolation.strip_tracking_params(
                "https://example.com/page?utm_source=google&utm_medium=cpc&id=123"
            ),
            "https://example.com/page?id=123"
        );
        
        // Should strip Facebook click ID
        assert_eq!(
            isolation.strip_tracking_params(
                "https://example.com/?fbclid=abc123"
            ),
            "https://example.com/"
        );
        
        // Should preserve non-tracking params
        assert_eq!(
            isolation.strip_tracking_params(
                "https://example.com/search?q=test&page=2"
            ),
            "https://example.com/search?q=test&page=2"
        );
    }

    #[test]
    fn test_cross_site_tracker_detection() {
        let isolation = TrackerIsolation::new();
        
        // Simulate a domain appearing on multiple sites
        isolation.record_third_party("site1.com", "tracker.com");
        isolation.record_third_party("site2.com", "tracker.com");
        isolation.record_third_party("site3.com", "tracker.com");
        
        assert!(isolation.is_cross_site_tracker("tracker.com"));
        
        // A domain on only one site is not a cross-site tracker
        isolation.record_third_party("site1.com", "cdn.com");
        assert!(!isolation.is_cross_site_tracker("cdn.com"));
    }
}
