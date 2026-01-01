//! Domain Blocklist - Curated list of ad/tracker domains
//!
//! This is intentionally a MINIMAL list of the worst offenders.
//! We're not trying to replace uBlock Origin's 300K+ rules.
//! Instead, we provide baseline protection that layers with extensions.

use std::collections::HashSet;
use super::ShieldLevel;

/// Reason why a request was blocked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Advertising network
    Advertising,
    /// Tracking/analytics
    Tracker,
    /// Malware/phishing
    Malware,
    /// Cryptomining
    Cryptominer,
    /// Tracking pixel
    TrackingPixel,
    /// Known fingerprinter
    Fingerprinting,
}

impl BlockReason {
    pub fn description(&self) -> &'static str {
        match self {
            BlockReason::Advertising => "Advertising network",
            BlockReason::Tracker => "Tracking/analytics",
            BlockReason::Malware => "Malware/phishing",
            BlockReason::Cryptominer => "Cryptomining script",
            BlockReason::TrackingPixel => "Tracking pixel",
            BlockReason::Fingerprinting => "Fingerprinting attempt",
        }
    }
}

/// Domain blocklist
pub struct BlockList {
    /// Advertising domains (Standard+)
    advertising: HashSet<&'static str>,
    /// Tracker domains (Standard+)
    trackers: HashSet<&'static str>,
    /// Malware domains (always blocked)
    malware: HashSet<&'static str>,
    /// Cryptominer domains (Standard+)
    cryptominers: HashSet<&'static str>,
    /// Fingerprinting domains (Strict only)
    fingerprinters: HashSet<&'static str>,
}

impl BlockList {
    pub fn new() -> Self {
        Self {
            advertising: Self::load_advertising_domains(),
            trackers: Self::load_tracker_domains(),
            malware: Self::load_malware_domains(),
            cryptominers: Self::load_cryptominer_domains(),
            fingerprinters: Self::load_fingerprinter_domains(),
        }
    }

    /// Check if a domain should be blocked
    pub fn check(&self, domain: &str, level: ShieldLevel) -> Option<BlockReason> {
        let domain = domain.to_lowercase();

        // Always block malware, regardless of level
        if self.matches_domain(&domain, &self.malware) {
            return Some(BlockReason::Malware);
        }

        if level == ShieldLevel::Off {
            return None;
        }

        // Standard and Strict: block ads, trackers, cryptominers
        if self.matches_domain(&domain, &self.advertising) {
            return Some(BlockReason::Advertising);
        }

        if self.matches_domain(&domain, &self.trackers) {
            return Some(BlockReason::Tracker);
        }

        if self.matches_domain(&domain, &self.cryptominers) {
            return Some(BlockReason::Cryptominer);
        }

        // Strict only: block fingerprinters
        if level == ShieldLevel::Strict {
            if self.matches_domain(&domain, &self.fingerprinters) {
                return Some(BlockReason::Fingerprinting);
            }
        }

        None
    }

    /// Check if a domain matches any in the set (including subdomains)
    fn matches_domain(&self, domain: &str, set: &HashSet<&'static str>) -> bool {
        // Exact match
        if set.contains(domain) {
            return true;
        }

        // Check if it's a subdomain of a blocked domain
        let mut parts: Vec<&str> = domain.split('.').collect();
        while parts.len() > 1 {
            parts.remove(0);
            let parent = parts.join(".");
            if set.contains(parent.as_str()) {
                return true;
            }
        }

        false
    }

    /// Get total domain count
    pub fn domain_count(&self) -> usize {
        self.advertising.len()
            + self.trackers.len()
            + self.malware.len()
            + self.cryptominers.len()
            + self.fingerprinters.len()
    }

    /// Top advertising networks - the most egregious ones
    fn load_advertising_domains() -> HashSet<&'static str> {
        [
            // Google Ads
            "googlesyndication.com",
            "doubleclick.net",
            "googleadservices.com",
            "google-analytics.com",
            "googletagmanager.com",
            "googletagservices.com",
            "pagead2.googlesyndication.com",

            // Facebook/Meta
            "facebook.net",
            "fbcdn.net",
            "connect.facebook.net",
            "pixel.facebook.com",

            // Amazon Ads
            "amazon-adsystem.com",
            "aax.amazon.com",

            // Microsoft Ads
            "ads.microsoft.com",
            "bat.bing.com",

            // Twitter/X
            "ads-twitter.com",
            "analytics.twitter.com",

            // Major ad networks
            "adnxs.com",
            "adsrvr.org",
            "advertising.com",
            "rubiconproject.com",
            "criteo.com",
            "criteo.net",
            "outbrain.com",
            "taboola.com",
            "zedo.com",
            "openx.net",
            "pubmatic.com",
            "bidswitch.net",
            "casalemedia.com",
            "sharethrough.com",
            "indexww.com",
            "triplelift.com",
            "yieldmo.com",
            "medianet.com",
            "media.net",

            // Popup/aggressive ads
            "popads.net",
            "popcash.net",
            "propellerads.com",
            "revcontent.com",
            "mgid.com",

            // Video ads
            "vidazoo.com",
            "spotxchange.com",
            "teads.tv",
        ]
        .into_iter()
        .collect()
    }

    /// Major trackers - analytics and surveillance
    fn load_tracker_domains() -> HashSet<&'static str> {
        [
            // Major analytics
            "hotjar.com",
            "mouseflow.com",
            "fullstory.com",
            "heap.io",
            "heapanalytics.com",
            "mixpanel.com",
            "amplitude.com",
            "segment.io",
            "segment.com",
            "mxpnl.com",
            "kissmetrics.com",
            "crazyegg.com",
            "luckyorange.com",
            "smartlook.com",
            "logrocket.com",

            // Session replay (privacy nightmare)
            "clarity.ms",
            "inspectlet.com",
            "sessioncam.com",

            // Cross-site tracking
            "bluekai.com",
            "exelator.com",
            "liveramp.com",
            "tapad.com",
            "lotame.com",
            "oracle.com/cx", // Oracle Data Cloud
            "rlcdn.com",
            "demdex.net",
            "krxd.net",
            "omtrdc.net",

            // Fingerprinting services
            "iovation.com",
            "threatmetrix.com",
            "maxmind.com",

            // Email/form tracking
            "getsitecontrol.com",
            "optinmonster.com",
            "sumo.com",
            "sumome.com",

            // Other trackers
            "scorecardresearch.com",
            "quantserve.com",
            "chartbeat.com",
            "newrelic.com",
            "nr-data.net",
            "parsely.com",
            "mparticle.com",

            // Social widgets (tracking)
            "addthis.com",
            "addtoany.com",
            "sharethis.com",
        ]
        .into_iter()
        .collect()
    }

    /// Known malware/phishing domains
    fn load_malware_domains() -> HashSet<&'static str> {
        [
            // These would be populated from threat intelligence
            // Keeping minimal for now - extensions handle this better
            "malware-check.com", // Placeholder
        ]
        .into_iter()
        .collect()
    }

    /// Cryptomining scripts
    fn load_cryptominer_domains() -> HashSet<&'static str> {
        [
            "coinhive.com",
            "coin-hive.com",
            "authedmine.com",
            "crypto-loot.com",
            "cryptoloot.pro",
            "minero.cc",
            "minr.pw",
            "coinerra.com",
            "mineralt.io",
            "webmine.pro",
            "ppoi.org",
            "cryptonight.wasm",
            "coinimp.com",
            "jsecoin.com",
            "cryptaloot.pro",
            "2giga.link",
            "hashfor.cash",
            "webminepool.com",
        ]
        .into_iter()
        .collect()
    }

    /// Known fingerprinting services (Strict mode only)
    fn load_fingerprinter_domains() -> HashSet<&'static str> {
        [
            // Canvas/WebGL fingerprinting
            "fingerprintjs.com",
            "fpjs.io",
            "browserleaks.com",

            // Device fingerprinting
            "castle.io",
            "seon.io",
            "sardine.ai",

            // Bot detection (often fingerprints)
            "perimeterx.net",
            "distilnetworks.com",
            "datadome.co",
            "imperva.com",
            "kasada.io",
        ]
        .into_iter()
        .collect()
    }
}

impl Default for BlockList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advertising_blocked() {
        let list = BlockList::new();

        assert_eq!(
            list.check("doubleclick.net", ShieldLevel::Standard),
            Some(BlockReason::Advertising)
        );

        // Subdomain should also be blocked
        assert_eq!(
            list.check("ad.doubleclick.net", ShieldLevel::Standard),
            Some(BlockReason::Advertising)
        );
    }

    #[test]
    fn test_tracker_blocked() {
        let list = BlockList::new();

        assert_eq!(
            list.check("hotjar.com", ShieldLevel::Standard),
            Some(BlockReason::Tracker)
        );
    }

    #[test]
    fn test_cryptominer_blocked() {
        let list = BlockList::new();

        assert_eq!(
            list.check("coinhive.com", ShieldLevel::Standard),
            Some(BlockReason::Cryptominer)
        );
    }

    #[test]
    fn test_fingerprinter_strict_only() {
        let list = BlockList::new();

        // Should not block in Standard mode
        assert_eq!(
            list.check("fingerprintjs.com", ShieldLevel::Standard),
            None
        );

        // Should block in Strict mode
        assert_eq!(
            list.check("fingerprintjs.com", ShieldLevel::Strict),
            Some(BlockReason::Fingerprinting)
        );
    }

    #[test]
    fn test_off_mode() {
        let list = BlockList::new();

        // Only malware blocked when off
        assert_eq!(
            list.check("doubleclick.net", ShieldLevel::Off),
            None
        );
    }
}
