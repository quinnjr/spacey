//! AMO (addons.mozilla.org) Client
//!
//! Provides integration with Firefox's extension marketplace.
//! Supports searching, downloading, and installing extensions.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Base URL for AMO API v5
const AMO_API_BASE: &str = "https://addons.mozilla.org/api/v5";

/// AMO API client
pub struct AmoClient {
    client: reqwest::blocking::Client,
}

impl AmoClient {
    /// Create a new AMO client
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("SpaceyBrowser/1.0 (Firefox-compatible)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Search for extensions
    pub fn search(&self, query: &str, page: u32, page_size: u32) -> Result<SearchResults, AmoError> {
        let url = format!(
            "{}/addons/search/?q={}&page={}&page_size={}&type=extension&app=firefox",
            AMO_API_BASE,
            urlencoding::encode(query),
            page,
            page_size
        );

        let response = self.client.get(&url)
            .send()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmoError::ApiError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        response.json::<SearchResults>()
            .map_err(|e| AmoError::ParseError(e.to_string()))
    }

    /// Get addon details by ID or slug
    pub fn get_addon(&self, id_or_slug: &str) -> Result<AddonDetail, AmoError> {
        let url = format!("{}/addons/addon/{}/", AMO_API_BASE, id_or_slug);

        let response = self.client.get(&url)
            .send()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmoError::ApiError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        response.json::<AddonDetail>()
            .map_err(|e| AmoError::ParseError(e.to_string()))
    }

    /// Get featured extensions
    pub fn get_featured(&self, page_size: u32) -> Result<SearchResults, AmoError> {
        let url = format!(
            "{}/addons/search/?featured=true&page_size={}&type=extension&app=firefox&sort=recommended",
            AMO_API_BASE,
            page_size
        );

        let response = self.client.get(&url)
            .send()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmoError::ApiError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        response.json::<SearchResults>()
            .map_err(|e| AmoError::ParseError(e.to_string()))
    }

    /// Get popular extensions by category
    pub fn get_by_category(&self, category: &str, page_size: u32) -> Result<SearchResults, AmoError> {
        let url = format!(
            "{}/addons/search/?category={}&page_size={}&type=extension&app=firefox&sort=users",
            AMO_API_BASE,
            urlencoding::encode(category),
            page_size
        );

        let response = self.client.get(&url)
            .send()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmoError::ApiError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        response.json::<SearchResults>()
            .map_err(|e| AmoError::ParseError(e.to_string()))
    }

    /// Download an extension XPI file
    pub fn download_xpi(&self, addon: &AddonDetail, target_path: &Path) -> Result<(), AmoError> {
        let file = addon.current_version.as_ref()
            .and_then(|v| v.file.as_ref())
            .ok_or_else(|| AmoError::NoDownload)?;

        let response = self.client.get(&file.url)
            .send()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmoError::DownloadFailed(format!(
                "Download returned status {}",
                response.status()
            )));
        }

        let bytes = response.bytes()
            .map_err(|e| AmoError::NetworkError(e.to_string()))?;

        std::fs::write(target_path, bytes)
            .map_err(|e| AmoError::IoError(e.to_string()))?;

        log::info!("Downloaded {} to {:?}", addon.name, target_path);
        Ok(())
    }

    /// Get the recommended content blockers (like uBlock Origin)
    pub fn get_recommended_blockers(&self) -> Result<Vec<AddonSummary>, AmoError> {
        // uBlock Origin and other recommended content blockers
        let blockers = vec![
            "ublock-origin",
            "privacy-badger17",
            "decentraleyes",
            "clearurls",
            "canvasblocker",
        ];

        let mut results = Vec::new();
        
        for slug in blockers {
            match self.get_addon(slug) {
                Ok(addon) => results.push(addon.into_summary()),
                Err(e) => log::warn!("Failed to get {}: {}", slug, e),
            }
        }

        Ok(results)
    }
}

impl Default for AmoClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Search results from AMO
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResults {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<AddonSummary>,
}

/// Summary of an addon (from search results)
#[derive(Debug, Clone, Deserialize)]
pub struct AddonSummary {
    pub id: u64,
    pub name: LocalizedString,
    pub slug: String,
    pub summary: Option<LocalizedString>,
    pub url: String,
    pub icon_url: Option<String>,
    pub current_version: Option<VersionSummary>,
    pub ratings: Option<Ratings>,
    pub average_daily_users: Option<u64>,
    #[serde(default)]
    pub categories: Categories,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

/// Detailed addon information
#[derive(Debug, Clone, Deserialize)]
pub struct AddonDetail {
    pub id: u64,
    pub name: LocalizedString,
    pub slug: String,
    pub description: Option<LocalizedString>,
    pub summary: Option<LocalizedString>,
    pub url: String,
    pub icon_url: Option<String>,
    pub current_version: Option<Version>,
    pub ratings: Option<Ratings>,
    pub average_daily_users: Option<u64>,
    pub weekly_downloads: Option<u64>,
    pub homepage: Option<LocalizedString>,
    pub support_url: Option<LocalizedString>,
    pub contributions_url: Option<String>,
    #[serde(default)]
    pub categories: Categories,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub last_updated: Option<String>,
    pub created: Option<String>,
}

impl AddonDetail {
    /// Convert to summary
    pub fn into_summary(&self) -> AddonSummary {
        AddonSummary {
            id: self.id,
            name: self.name.clone(),
            slug: self.slug.clone(),
            summary: self.summary.clone(),
            url: self.url.clone(),
            icon_url: self.icon_url.clone(),
            current_version: self.current_version.as_ref().map(|v| VersionSummary {
                id: v.id,
                version: v.version.clone(),
            }),
            ratings: self.ratings.clone(),
            average_daily_users: self.average_daily_users,
            categories: self.categories.clone(),
            tags: self.tags.clone(),
        }
    }
}

/// Localized string (can be just a string or language map)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LocalizedString {
    Simple(String),
    Localized(std::collections::HashMap<String, String>),
}

impl LocalizedString {
    /// Get the string value (preferring en-US)
    pub fn get(&self) -> &str {
        match self {
            LocalizedString::Simple(s) => s,
            LocalizedString::Localized(map) => {
                map.get("en-US")
                    .or_else(|| map.get("en"))
                    .or_else(|| map.values().next())
                    .map(|s| s.as_str())
                    .unwrap_or("")
            }
        }
    }
}

impl std::fmt::Display for LocalizedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// Version summary (from search results)
#[derive(Debug, Clone, Deserialize)]
pub struct VersionSummary {
    pub id: u64,
    pub version: String,
}

/// Full version information
#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    pub id: u64,
    pub version: String,
    pub file: Option<FileInfo>,
    pub compatibility: Option<Compatibility>,
    pub release_notes: Option<LocalizedString>,
}

/// File download information
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: u64,
    pub url: String,
    pub size: u64,
    pub hash: String,
    pub status: String,
}

/// Compatibility information
#[derive(Debug, Clone, Deserialize)]
pub struct Compatibility {
    pub firefox: Option<VersionRange>,
    pub android: Option<VersionRange>,
}

/// Version range for compatibility
#[derive(Debug, Clone, Deserialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
}

/// Ratings information
#[derive(Debug, Clone, Deserialize)]
pub struct Ratings {
    pub average: f32,
    pub count: u32,
}

/// Categories
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Categories {
    #[serde(default)]
    pub firefox: Vec<String>,
    #[serde(default)]
    pub android: Vec<String>,
}

/// Tag
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub tag_text: String,
}

/// Author information
#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub id: u64,
    pub name: String,
    pub url: Option<String>,
    pub username: Option<String>,
}

/// Well-known addon categories
pub mod categories {
    pub const ALERTS_UPDATES: &str = "alerts-updates";
    pub const APPEARANCE: &str = "appearance";
    pub const BOOKMARKS: &str = "bookmarks";
    pub const DOWNLOAD_MANAGEMENT: &str = "download-management";
    pub const FEEDS_NEWS_BLOGGING: &str = "feeds-news-blogging";
    pub const GAMES_ENTERTAINMENT: &str = "games-entertainment";
    pub const LANGUAGE_SUPPORT: &str = "language-support";
    pub const PHOTOS_MUSIC_VIDEOS: &str = "photos-music-videos";
    pub const PRIVACY_SECURITY: &str = "privacy-security";
    pub const SEARCH_TOOLS: &str = "search-tools";
    pub const SHOPPING: &str = "shopping";
    pub const SOCIAL_COMMUNICATION: &str = "social-communication";
    pub const TABS: &str = "tabs";
    pub const WEB_DEVELOPMENT: &str = "web-development";
    pub const OTHER: &str = "other";
}

/// AMO API errors
#[derive(Debug)]
pub enum AmoError {
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    NoDownload,
    DownloadFailed(String),
    IoError(String),
}

impl std::fmt::Display for AmoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmoError::NetworkError(e) => write!(f, "Network error: {}", e),
            AmoError::ApiError(e) => write!(f, "API error: {}", e),
            AmoError::ParseError(e) => write!(f, "Parse error: {}", e),
            AmoError::NoDownload => write!(f, "No download available"),
            AmoError::DownloadFailed(e) => write!(f, "Download failed: {}", e),
            AmoError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AmoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_string() {
        let simple = LocalizedString::Simple("Hello".to_string());
        assert_eq!(simple.get(), "Hello");

        let mut map = std::collections::HashMap::new();
        map.insert("en-US".to_string(), "Hello".to_string());
        map.insert("es".to_string(), "Hola".to_string());
        let localized = LocalizedString::Localized(map);
        assert_eq!(localized.get(), "Hello");
    }

    // Note: Network tests should be run manually
    #[test]
    #[ignore]
    fn test_search() {
        let client = AmoClient::new();
        let results = client.search("ublock", 1, 10).unwrap();
        assert!(results.count > 0);
    }

    #[test]
    #[ignore]
    fn test_get_ublock_origin() {
        let client = AmoClient::new();
        let addon = client.get_addon("ublock-origin").unwrap();
        assert!(addon.name.get().contains("uBlock"));
    }
}
