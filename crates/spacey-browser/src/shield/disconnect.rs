//! Disconnect List Integration
//!
//! Browser-specific implementation of the Disconnect tracking protection list.
//! Based on the Disconnect.me open-source tracker list used by Firefox's
//! Enhanced Tracking Protection.
//!
//! The Disconnect list categorizes trackers into several types:
//! - **Advertising**: Ad networks and ad-related tracking
//! - **Analytics**: Analytics and measurement services
//! - **Social**: Social media tracking widgets and APIs
//! - **Fingerprinting**: Browser fingerprinting services
//! - **Cryptomining**: Cryptocurrency mining scripts
//! - **Content**: Tracking CDNs (blocked in strict mode only)
//!
//! ## Credits
//!
//! The Disconnect list is maintained by Disconnect.me and used under the
//! GNU General Public License v3.0. See: https://github.com/nicknockname/disconnect-tracking-protection
//!
//! ## Note on Completeness
//!
//! This is a curated subset optimized for browser integration.
//! For the complete list, see the Disconnect GitHub repository.

use std::collections::{HashMap, HashSet};
use super::ShieldLevel;

/// Disconnect tracker categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisconnectCategory {
    /// Advertising networks
    Advertising,
    /// Analytics and measurement
    Analytics,
    /// Social media widgets
    Social,
    /// Fingerprinting services
    Fingerprinting,
    /// Cryptomining scripts
    Cryptomining,
    /// Content delivery (can break sites)
    Content,
    /// Disconnect's own curated list
    Disconnect,
}

impl DisconnectCategory {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            DisconnectCategory::Advertising => "Advertising",
            DisconnectCategory::Analytics => "Analytics",
            DisconnectCategory::Social => "Social",
            DisconnectCategory::Fingerprinting => "Fingerprinting",
            DisconnectCategory::Cryptomining => "Cryptomining",
            DisconnectCategory::Content => "Content",
            DisconnectCategory::Disconnect => "Disconnect",
        }
    }
    
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            DisconnectCategory::Advertising => "Advertising networks and ad-related tracking",
            DisconnectCategory::Analytics => "Analytics and measurement services",
            DisconnectCategory::Social => "Social media tracking widgets",
            DisconnectCategory::Fingerprinting => "Browser fingerprinting services",
            DisconnectCategory::Cryptomining => "Cryptocurrency mining scripts",
            DisconnectCategory::Content => "Tracking CDNs (may break some sites)",
            DisconnectCategory::Disconnect => "Disconnect's curated tracker list",
        }
    }
    
    /// Whether this category is blocked by default in Standard mode
    pub fn blocked_in_standard(&self) -> bool {
        match self {
            DisconnectCategory::Advertising => true,
            DisconnectCategory::Analytics => true,
            DisconnectCategory::Social => true,
            DisconnectCategory::Fingerprinting => false, // Strict only
            DisconnectCategory::Cryptomining => true,
            DisconnectCategory::Content => false, // Strict only
            DisconnectCategory::Disconnect => true,
        }
    }
}

/// A tracking service from the Disconnect list
#[derive(Debug, Clone)]
pub struct TrackerService {
    /// Service name (e.g., "Google", "Facebook")
    pub name: String,
    /// Primary domain
    pub primary_domain: String,
    /// All domains associated with this service
    pub domains: HashSet<String>,
    /// Category
    pub category: DisconnectCategory,
}

/// The Disconnect tracking protection list
pub struct DisconnectList {
    /// All tracker services
    services: Vec<TrackerService>,
    /// Quick lookup by domain -> category
    domain_lookup: HashMap<String, DisconnectCategory>,
    /// Domains by category for stats
    category_counts: HashMap<DisconnectCategory, usize>,
}

impl DisconnectList {
    /// Create a new Disconnect list with embedded data
    pub fn new() -> Self {
        let mut list = Self {
            services: Vec::new(),
            domain_lookup: HashMap::new(),
            category_counts: HashMap::new(),
        };
        
        // Load all categories
        list.load_advertising();
        list.load_analytics();
        list.load_social();
        list.load_fingerprinting();
        list.load_cryptomining();
        list.load_content();
        list.load_disconnect_curated();
        
        // Update category counts
        for category in [
            DisconnectCategory::Advertising,
            DisconnectCategory::Analytics,
            DisconnectCategory::Social,
            DisconnectCategory::Fingerprinting,
            DisconnectCategory::Cryptomining,
            DisconnectCategory::Content,
            DisconnectCategory::Disconnect,
        ] {
            let count = list.domain_lookup.values().filter(|c| **c == category).count();
            list.category_counts.insert(category, count);
        }
        
        list
    }
    
    /// Check if a domain is in the Disconnect list
    pub fn check(&self, domain: &str, level: ShieldLevel) -> Option<DisconnectCategory> {
        let domain_lower = domain.to_lowercase();
        
        // Direct lookup
        if let Some(category) = self.domain_lookup.get(&domain_lower) {
            if self.should_block_category(*category, level) {
                return Some(*category);
            }
        }
        
        // Check parent domains
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len().saturating_sub(1) {
            let parent = parts[i..].join(".");
            if let Some(category) = self.domain_lookup.get(&parent) {
                if self.should_block_category(*category, level) {
                    return Some(*category);
                }
            }
        }
        
        None
    }
    
    /// Check if a category should be blocked at this level
    fn should_block_category(&self, category: DisconnectCategory, level: ShieldLevel) -> bool {
        match level {
            ShieldLevel::Off => false,
            ShieldLevel::Standard => category.blocked_in_standard(),
            ShieldLevel::Strict => true, // Block everything in strict mode
        }
    }
    
    /// Get total domain count
    pub fn domain_count(&self) -> usize {
        self.domain_lookup.len()
    }
    
    /// Get count for a specific category
    pub fn category_count(&self, category: DisconnectCategory) -> usize {
        *self.category_counts.get(&category).unwrap_or(&0)
    }
    
    /// Get all services
    pub fn services(&self) -> &[TrackerService] {
        &self.services
    }
    
    /// Add a service to the list
    fn add_service(&mut self, name: &str, primary: &str, domains: &[&str], category: DisconnectCategory) {
        let domain_set: HashSet<String> = domains.iter().map(|d| d.to_lowercase()).collect();
        
        // Add to lookup
        for domain in &domain_set {
            self.domain_lookup.insert(domain.clone(), category);
        }
        self.domain_lookup.insert(primary.to_lowercase(), category);
        
        let mut all_domains = domain_set;
        all_domains.insert(primary.to_lowercase());
        
        self.services.push(TrackerService {
            name: name.to_string(),
            primary_domain: primary.to_lowercase(),
            domains: all_domains,
            category,
        });
    }

    /// Load advertising trackers from Disconnect list
    fn load_advertising(&mut self) {
        // Google Ads
        self.add_service("Google Ads", "google.com", &[
            "doubleclick.net",
            "googlesyndication.com",
            "googleadservices.com",
            "googletagmanager.com",
            "googletagservices.com",
            "2mdn.net",
            "admob.com",
            "adsense.com",
            "adsensecustomsearchads.com",
            "adwords.com",
            "googlevideo.com", // Video ads
        ], DisconnectCategory::Advertising);
        
        // Facebook/Meta Ads
        self.add_service("Facebook Ads", "facebook.com", &[
            "facebook.net",
            "fbcdn.net",
            "fb.com",
            "fbsbx.com",
            "instagram.com", // Meta ad network
            "atdmt.com",
            "liverail.com",
            "atlassolutions.com",
        ], DisconnectCategory::Advertising);
        
        // Microsoft/LinkedIn Ads
        self.add_service("Microsoft Ads", "microsoft.com", &[
            "ads.microsoft.com",
            "adnxs.com",
            "appnexus.com",
            "bat.bing.com",
            "linkedin.com", // LinkedIn Ads
        ], DisconnectCategory::Advertising);
        
        // Amazon Ads
        self.add_service("Amazon Ads", "amazon.com", &[
            "amazon-adsystem.com",
            "assoc-amazon.com",
            "amazonservices.com",
            "serving-sys.com",
            "sizmek.com",
        ], DisconnectCategory::Advertising);
        
        // Twitter/X Ads
        self.add_service("Twitter Ads", "twitter.com", &[
            "ads-twitter.com",
            "t.co",
            "twimg.com",
            "mopub.com",
        ], DisconnectCategory::Advertising);
        
        // Trade Desk
        self.add_service("The Trade Desk", "thetradedesk.com", &[
            "adsrvr.org",
            "ttdns.com",
        ], DisconnectCategory::Advertising);
        
        // Criteo
        self.add_service("Criteo", "criteo.com", &[
            "criteo.net",
            "hlserve.com",
            "emailretargeting.com",
        ], DisconnectCategory::Advertising);
        
        // OpenX
        self.add_service("OpenX", "openx.com", &[
            "openx.net",
            "servedbyopenx.com",
            "openxenterprise.com",
        ], DisconnectCategory::Advertising);
        
        // Rubicon Project
        self.add_service("Rubicon Project", "rubiconproject.com", &[
            "rubiconproject.net",
            "chango.com",
            "optimera.nyc",
        ], DisconnectCategory::Advertising);
        
        // PubMatic
        self.add_service("PubMatic", "pubmatic.com", &[
            "pubmatic.net",
            "vertamedia.com",
        ], DisconnectCategory::Advertising);
        
        // Taboola
        self.add_service("Taboola", "taboola.com", &[
            "taboola.map.fastly.net",
            "taboolasyndication.com",
        ], DisconnectCategory::Advertising);
        
        // Outbrain
        self.add_service("Outbrain", "outbrain.com", &[
            "outbrainimg.com",
            "widgets.outbrain.com",
        ], DisconnectCategory::Advertising);
        
        // Index Exchange
        self.add_service("Index Exchange", "indexexchange.com", &[
            "casalemedia.com",
            "indexww.com",
        ], DisconnectCategory::Advertising);
        
        // MGID
        self.add_service("MGID", "mgid.com", &[
            "dt07.net",
            "lentainform.com",
        ], DisconnectCategory::Advertising);
        
        // Propeller Ads
        self.add_service("PropellerAds", "propellerads.com", &[
            "propellerpops.com",
            "propu.sh",
        ], DisconnectCategory::Advertising);
        
        // PopAds
        self.add_service("PopAds", "popads.net", &[
            "popcash.net",
            "popunder.net",
        ], DisconnectCategory::Advertising);
        
        // Media.net
        self.add_service("Media.net", "media.net", &[
            "medianet.com",
            "contextual.media.net",
        ], DisconnectCategory::Advertising);
        
        // Verizon Media
        self.add_service("Verizon Media", "verizonmedia.com", &[
            "advertising.com",
            "oath.com",
            "yahoo.com", // Yahoo Ads portion
            "gemini.yahoo.com",
            "brightroll.com",
            "flurry.com",
        ], DisconnectCategory::Advertising);
        
        // AdColony
        self.add_service("AdColony", "adcolony.com", &[
            "adc3-launch.adcolony.com",
            "ads.adcolony.com",
        ], DisconnectCategory::Advertising);
        
        // Unity Ads
        self.add_service("Unity Ads", "unity3d.com", &[
            "unityads.unity3d.com",
            "config.uca.cloud.unity3d.com",
        ], DisconnectCategory::Advertising);
        
        // IronSource
        self.add_service("ironSource", "ironsrc.com", &[
            "supersonic.com",
            "supersonicads.com",
        ], DisconnectCategory::Advertising);
        
        // Vungle
        self.add_service("Vungle", "vungle.com", &[
            "ads.vungle.com",
            "vungleads.com",
        ], DisconnectCategory::Advertising);
        
        // AppLovin
        self.add_service("AppLovin", "applovin.com", &[
            "applvn.com",
            "pxl.applovin.com",
        ], DisconnectCategory::Advertising);
        
        // InMobi
        self.add_service("InMobi", "inmobi.com", &[
            "api.w.inmobi.com",
            "config.inmobi.com",
        ], DisconnectCategory::Advertising);
        
        // Smaato
        self.add_service("Smaato", "smaato.net", &[
            "smaato.com",
            "soma.smaato.net",
        ], DisconnectCategory::Advertising);
        
        // Digital Turbine
        self.add_service("Digital Turbine", "digitalturbine.com", &[
            "appia.com",
            "fyber.com",
        ], DisconnectCategory::Advertising);
        
        // TripleLift
        self.add_service("TripleLift", "triplelift.com", &[
            "3lift.com",
            "tlx.3lift.com",
        ], DisconnectCategory::Advertising);
        
        // Sharethrough
        self.add_service("Sharethrough", "sharethrough.com", &[
            "stg.sharethrough.com",
        ], DisconnectCategory::Advertising);
        
        // 33Across
        self.add_service("33Across", "33across.com", &[
            "tynt.com",
            "33across.net",
        ], DisconnectCategory::Advertising);
        
        // Sovrn
        self.add_service("Sovrn", "sovrn.com", &[
            "lijit.com",
            "viglink.com",
        ], DisconnectCategory::Advertising);
        
        // GumGum
        self.add_service("GumGum", "gumgum.com", &[
            "g2.gumgum.com",
            "pixel.gumgum.com",
        ], DisconnectCategory::Advertising);
        
        // Teads
        self.add_service("Teads", "teads.tv", &[
            "teads.com",
            "a.teads.tv",
        ], DisconnectCategory::Advertising);
        
        // SpotX
        self.add_service("SpotX", "spotxchange.com", &[
            "spotx.tv",
            "spotxcdn.com",
        ], DisconnectCategory::Advertising);
    }

    /// Load analytics trackers
    fn load_analytics(&mut self) {
        // Google Analytics
        self.add_service("Google Analytics", "google-analytics.com", &[
            "analytics.google.com",
            "googleanalytics.com",
            "urchin.com",
        ], DisconnectCategory::Analytics);
        
        // Adobe Analytics
        self.add_service("Adobe Analytics", "adobe.com", &[
            "2o7.net",
            "omtrdc.net",
            "demdex.net",
            "adobedtm.com",
            "omniture.com",
            "adobedc.net",
        ], DisconnectCategory::Analytics);
        
        // Hotjar
        self.add_service("Hotjar", "hotjar.com", &[
            "hotjar.io",
            "static.hotjar.com",
            "script.hotjar.com",
        ], DisconnectCategory::Analytics);
        
        // Mixpanel
        self.add_service("Mixpanel", "mixpanel.com", &[
            "mxpnl.com",
            "decide.mixpanel.com",
            "api.mixpanel.com",
        ], DisconnectCategory::Analytics);
        
        // Amplitude
        self.add_service("Amplitude", "amplitude.com", &[
            "api.amplitude.com",
            "cdn.amplitude.com",
        ], DisconnectCategory::Analytics);
        
        // Segment
        self.add_service("Segment", "segment.com", &[
            "segment.io",
            "api.segment.io",
            "cdn.segment.com",
        ], DisconnectCategory::Analytics);
        
        // Heap
        self.add_service("Heap Analytics", "heap.io", &[
            "heapanalytics.com",
            "cdn.heapanalytics.com",
        ], DisconnectCategory::Analytics);
        
        // Mouseflow
        self.add_service("Mouseflow", "mouseflow.com", &[
            "cdn.mouseflow.com",
            "api.mouseflow.com",
        ], DisconnectCategory::Analytics);
        
        // FullStory
        self.add_service("FullStory", "fullstory.com", &[
            "rs.fullstory.com",
            "edge.fullstory.com",
        ], DisconnectCategory::Analytics);
        
        // LogRocket
        self.add_service("LogRocket", "logrocket.com", &[
            "lr-intake.com",
            "lr-in.com",
        ], DisconnectCategory::Analytics);
        
        // Crazy Egg
        self.add_service("Crazy Egg", "crazyegg.com", &[
            "cetrk.com",
            "dnnlab.crazyegg.com",
        ], DisconnectCategory::Analytics);
        
        // Lucky Orange
        self.add_service("Lucky Orange", "luckyorange.com", &[
            "luckyorange.net",
            "cdn.luckyorange.com",
        ], DisconnectCategory::Analytics);
        
        // Smartlook
        self.add_service("Smartlook", "smartlook.com", &[
            "rec.smartlook.com",
            "web-sdk.smartlook.com",
        ], DisconnectCategory::Analytics);
        
        // Microsoft Clarity
        self.add_service("Microsoft Clarity", "clarity.ms", &[
            "c.clarity.ms",
            "d.clarity.ms",
        ], DisconnectCategory::Analytics);
        
        // Inspectlet
        self.add_service("Inspectlet", "inspectlet.com", &[
            "cdn.inspectlet.com",
            "hn.inspectlet.com",
        ], DisconnectCategory::Analytics);
        
        // Chartbeat
        self.add_service("Chartbeat", "chartbeat.com", &[
            "chartbeat.net",
            "static.chartbeat.com",
        ], DisconnectCategory::Analytics);
        
        // Comscore
        self.add_service("Comscore", "comscore.com", &[
            "scorecardresearch.com",
            "sbtechny498.com",
        ], DisconnectCategory::Analytics);
        
        // Quantcast
        self.add_service("Quantcast", "quantcast.com", &[
            "quantserve.com",
            "pixel.quantserve.com",
        ], DisconnectCategory::Analytics);
        
        // New Relic
        self.add_service("New Relic", "newrelic.com", &[
            "nr-data.net",
            "bam.nr-data.net",
            "js-agent.newrelic.com",
        ], DisconnectCategory::Analytics);
        
        // Parse.ly
        self.add_service("Parse.ly", "parsely.com", &[
            "parse.ly",
            "d1z2jf7jlzjs58.cloudfront.net",
        ], DisconnectCategory::Analytics);
        
        // Kissmetrics
        self.add_service("Kissmetrics", "kissmetrics.com", &[
            "kissmetricshq.com",
            "i.kissmetrics.com",
        ], DisconnectCategory::Analytics);
        
        // Pendo
        self.add_service("Pendo", "pendo.io", &[
            "cdn.pendo.io",
            "app.pendo.io",
        ], DisconnectCategory::Analytics);
        
        // mParticle
        self.add_service("mParticle", "mparticle.com", &[
            "identity.mparticle.com",
            "nativesdks.mparticle.com",
        ], DisconnectCategory::Analytics);
    }

    /// Load social media trackers
    fn load_social(&mut self) {
        // Facebook Social Plugins
        self.add_service("Facebook Social", "facebook.com", &[
            "connect.facebook.net",
            "staticxx.facebook.com",
            "pixel.facebook.com",
        ], DisconnectCategory::Social);
        
        // Twitter Widgets
        self.add_service("Twitter Social", "twitter.com", &[
            "platform.twitter.com",
            "syndication.twitter.com",
            "cdn.syndication.twimg.com",
        ], DisconnectCategory::Social);
        
        // LinkedIn Social
        self.add_service("LinkedIn Social", "linkedin.com", &[
            "platform.linkedin.com",
            "snap.licdn.com",
        ], DisconnectCategory::Social);
        
        // Pinterest
        self.add_service("Pinterest", "pinterest.com", &[
            "pinimg.com",
            "log.pinterest.com",
            "ct.pinterest.com",
        ], DisconnectCategory::Social);
        
        // TikTok
        self.add_service("TikTok", "tiktok.com", &[
            "byteoversea.com",
            "bytedance.com",
            "musical.ly",
            "analytics.tiktok.com",
        ], DisconnectCategory::Social);
        
        // Reddit
        self.add_service("Reddit", "reddit.com", &[
            "redditstatic.com",
            "redditmedia.com",
            "events.reddit.com",
        ], DisconnectCategory::Social);
        
        // Disqus
        self.add_service("Disqus", "disqus.com", &[
            "disquscdn.com",
            "referrer.disqus.com",
        ], DisconnectCategory::Social);
        
        // AddThis
        self.add_service("AddThis", "addthis.com", &[
            "addthiscdn.com",
            "addthisedge.com",
        ], DisconnectCategory::Social);
        
        // ShareThis
        self.add_service("ShareThis", "sharethis.com", &[
            "w.sharethis.com",
            "l.sharethis.com",
        ], DisconnectCategory::Social);
        
        // Snapchat
        self.add_service("Snapchat", "snapchat.com", &[
            "sc-static.net",
            "snap.com",
            "tr.snapchat.com",
        ], DisconnectCategory::Social);
        
        // Tumblr
        self.add_service("Tumblr", "tumblr.com", &[
            "t.umblr.com",
            "assets.tumblr.com",
        ], DisconnectCategory::Social);
    }

    /// Load fingerprinting services
    fn load_fingerprinting(&mut self) {
        // FingerprintJS
        self.add_service("FingerprintJS", "fingerprintjs.com", &[
            "fpjs.io",
            "api.fpjs.io",
            "cdn.fpjs.io",
        ], DisconnectCategory::Fingerprinting);
        
        // ThreatMetrix
        self.add_service("ThreatMetrix", "threatmetrix.com", &[
            "lexisnexis.com",
            "tmx.threatmetrix.com",
        ], DisconnectCategory::Fingerprinting);
        
        // iovation
        self.add_service("iovation", "iovation.com", &[
            "iesnare.com",
            "iesnare.net",
        ], DisconnectCategory::Fingerprinting);
        
        // Castle
        self.add_service("Castle", "castle.io", &[
            "api.castle.io",
            "cdn.castle.io",
        ], DisconnectCategory::Fingerprinting);
        
        // Seon
        self.add_service("Seon", "seon.io", &[
            "seondf.com",
            "cdn.seon.io",
        ], DisconnectCategory::Fingerprinting);
        
        // PerimeterX
        self.add_service("PerimeterX", "perimeterx.com", &[
            "perimeterx.net",
            "pxcdn.net",
            "px-cloud.net",
        ], DisconnectCategory::Fingerprinting);
        
        // Distil Networks
        self.add_service("Distil Networks", "distilnetworks.com", &[
            "distil.us",
            "arkoselabs.com",
        ], DisconnectCategory::Fingerprinting);
        
        // DataDome
        self.add_service("DataDome", "datadome.co", &[
            "ct.datadome.co",
            "js.datadome.co",
        ], DisconnectCategory::Fingerprinting);
        
        // Kasada
        self.add_service("Kasada", "kasada.io", &[
            "cd.kasadapolyform.io",
        ], DisconnectCategory::Fingerprinting);
        
        // Imperva
        self.add_service("Imperva", "imperva.com", &[
            "incapsula.com",
            "distil.mobi",
        ], DisconnectCategory::Fingerprinting);
        
        // BrowserLeaks (testing site, but used for fingerprinting)
        self.add_service("BrowserLeaks", "browserleaks.com", &[
            "browserleaks.net",
        ], DisconnectCategory::Fingerprinting);
        
        // MaxMind
        self.add_service("MaxMind", "maxmind.com", &[
            "geoip.maxmind.com",
            "mm_bcts.eproof.com",
        ], DisconnectCategory::Fingerprinting);
    }

    /// Load cryptomining scripts
    fn load_cryptomining(&mut self) {
        // Coinhive (defunct but domains still exist)
        self.add_service("Coinhive", "coinhive.com", &[
            "coin-hive.com",
            "authedmine.com",
            "coinhive-proxy.com",
        ], DisconnectCategory::Cryptomining);
        
        // CryptoLoot
        self.add_service("CryptoLoot", "crypto-loot.com", &[
            "cryptoloot.pro",
            "cryptaloot.pro",
        ], DisconnectCategory::Cryptomining);
        
        // JSEcoin
        self.add_service("JSEcoin", "jsecoin.com", &[
            "load.jsecoin.com",
            "server.jsecoin.com",
        ], DisconnectCategory::Cryptomining);
        
        // CoinImp
        self.add_service("CoinImp", "coinimp.com", &[
            "static.coinimp.com",
            "www.coinimp.net",
        ], DisconnectCategory::Cryptomining);
        
        // Minero
        self.add_service("Minero", "minero.cc", &[
            "api.minero.cc",
            "static.minero.cc",
        ], DisconnectCategory::Cryptomining);
        
        // Others
        self.add_service("WebMinerPool", "webminepool.com", &[
            "pool.webminepool.com",
        ], DisconnectCategory::Cryptomining);
        
        self.add_service("Mineralt", "mineralt.io", &[
            "s.mineralt.io",
        ], DisconnectCategory::Cryptomining);
        
        self.add_service("Minr", "minr.pw", &[
            "cdn.minr.pw",
            "minr.me",
        ], DisconnectCategory::Cryptomining);
        
        self.add_service("PPOI", "ppoi.org", &[
            "papoto.com",
        ], DisconnectCategory::Cryptomining);
    }

    /// Load content delivery trackers (strict mode only)
    fn load_content(&mut self) {
        // Note: These are blocked only in strict mode as they may break sites
        
        // Google CDN (used for fonts, etc)
        // Not blocking google.com or googleapis.com as too many sites depend on them
        
        // Cloudflare Analytics (separate from CDN)
        self.add_service("Cloudflare Analytics", "cloudflare.com", &[
            "static.cloudflareinsights.com",
        ], DisconnectCategory::Content);
        
        // Akamai Analytics
        self.add_service("Akamai Analytics", "akamai.net", &[
            "o.akamaihd.net",  // Analytics portion
        ], DisconnectCategory::Content);
        
        // Fastly Analytics
        self.add_service("Fastly Analytics", "fastly.net", &[
            "stats.g.fastly.net",
        ], DisconnectCategory::Content);
    }

    /// Load Disconnect's own curated list
    fn load_disconnect_curated(&mut self) {
        // BlueKai (Oracle)
        self.add_service("BlueKai", "bluekai.com", &[
            "bkrtx.com",
            "tags.bluekai.com",
        ], DisconnectCategory::Disconnect);
        
        // Exelator
        self.add_service("Exelator", "exelator.com", &[
            "load.exelator.com",
        ], DisconnectCategory::Disconnect);
        
        // LiveRamp
        self.add_service("LiveRamp", "liveramp.com", &[
            "rlcdn.com",
            "pippio.com",
        ], DisconnectCategory::Disconnect);
        
        // Tapad
        self.add_service("Tapad", "tapad.com", &[
            "tapestry.tapad.com",
        ], DisconnectCategory::Disconnect);
        
        // Lotame
        self.add_service("Lotame", "lotame.com", &[
            "crwdcntrl.net",
            "ad.crwdcntrl.net",
        ], DisconnectCategory::Disconnect);
        
        // Krux (Salesforce)
        self.add_service("Krux", "krux.com", &[
            "krxd.net",
            "beacon.krxd.net",
        ], DisconnectCategory::Disconnect);
        
        // Acxiom
        self.add_service("Acxiom", "acxiom.com", &[
            "acs86.com",
            "axciom.com",
        ], DisconnectCategory::Disconnect);
        
        // Neustar
        self.add_service("Neustar", "neustar.biz", &[
            "agkn.com",
            "adadvisor.net",
        ], DisconnectCategory::Disconnect);
        
        // Oracle Data Cloud
        self.add_service("Oracle Data Cloud", "oracle.com", &[
            "addthis.com",  // Owned by Oracle
            "moat.com",
            "grapeshot.com",
        ], DisconnectCategory::Disconnect);
        
        // Drawbridge
        self.add_service("Drawbridge", "drawbridge.com", &[
            "adsymptotic.com",
        ], DisconnectCategory::Disconnect);
        
        // The Nielsen Company
        self.add_service("Nielsen", "nielsen.com", &[
            "imrworldwide.com",
            "exelate.com",
        ], DisconnectCategory::Disconnect);
        
        // comScore
        self.add_service("comScore DMP", "comscore.com", &[
            "scorecardresearch.com",
            "voicefive.com",
        ], DisconnectCategory::Disconnect);
        
        // Epsilon
        self.add_service("Epsilon", "epsilon.com", &[
            "conversantmedia.com",
            "dotomi.com",
        ], DisconnectCategory::Disconnect);
    }
}

impl Default for DisconnectList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_disconnect_list_creation() {
        let list = DisconnectList::new();
        assert!(list.domain_count() > 100, "Should have many domains");
    }
    
    #[test]
    fn test_advertising_blocked() {
        let list = DisconnectList::new();
        
        assert_eq!(
            list.check("doubleclick.net", ShieldLevel::Standard),
            Some(DisconnectCategory::Advertising)
        );
        
        assert_eq!(
            list.check("ad.doubleclick.net", ShieldLevel::Standard),
            Some(DisconnectCategory::Advertising)
        );
    }
    
    #[test]
    fn test_analytics_blocked() {
        let list = DisconnectList::new();
        
        assert_eq!(
            list.check("hotjar.com", ShieldLevel::Standard),
            Some(DisconnectCategory::Analytics)
        );
    }
    
    #[test]
    fn test_social_blocked() {
        let list = DisconnectList::new();
        
        assert_eq!(
            list.check("connect.facebook.net", ShieldLevel::Standard),
            Some(DisconnectCategory::Social)
        );
    }
    
    #[test]
    fn test_fingerprinting_strict_only() {
        let list = DisconnectList::new();
        
        // Not blocked in standard
        assert_eq!(
            list.check("fingerprintjs.com", ShieldLevel::Standard),
            None
        );
        
        // Blocked in strict
        assert_eq!(
            list.check("fingerprintjs.com", ShieldLevel::Strict),
            Some(DisconnectCategory::Fingerprinting)
        );
    }
    
    #[test]
    fn test_cryptomining_blocked() {
        let list = DisconnectList::new();
        
        assert_eq!(
            list.check("coinhive.com", ShieldLevel::Standard),
            Some(DisconnectCategory::Cryptomining)
        );
    }
    
    #[test]
    fn test_off_mode() {
        let list = DisconnectList::new();
        
        assert_eq!(
            list.check("doubleclick.net", ShieldLevel::Off),
            None
        );
    }
    
    #[test]
    fn test_category_counts() {
        let list = DisconnectList::new();
        
        assert!(list.category_count(DisconnectCategory::Advertising) > 20);
        assert!(list.category_count(DisconnectCategory::Analytics) > 10);
        assert!(list.category_count(DisconnectCategory::Social) > 5);
    }
}
