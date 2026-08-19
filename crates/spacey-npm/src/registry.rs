//! NPM Registry client with async support.

use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument};

use crate::error::{Result, SnpmError};
use crate::package::{PackageVersion, RegistryPackage};

/// Default NPM registry URL.
pub const DEFAULT_REGISTRY: &str = "https://registry.npmjs.org";

/// NPM Registry client.
#[derive(Clone)]
pub struct RegistryClient {
    client: Client,
    registry_url: String,
    /// Cache for package metadata
    cache: Arc<dashmap::DashMap<String, RegistryPackage>>,
}

impl RegistryClient {
    /// Create a new registry client.
    pub fn new(registry_url: Option<&str>, insecure: bool) -> Result<Self> {
        let registry_url = registry_url
            .map(|s| s.trim_end_matches('/').to_string())
            .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());

        let mut builder = Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(20)
            .user_agent(format!("snpm/{}", env!("CARGO_PKG_VERSION")));

        if insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            registry_url,
            cache: Arc::new(dashmap::DashMap::new()),
        })
    }

    /// Get the registry URL.
    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }

    /// Fetch package metadata from the registry.
    #[instrument(skip(self))]
    pub async fn get_package(&self, name: &str) -> Result<RegistryPackage> {
        // Check cache first
        if let Some(pkg) = self.cache.get(name) {
            debug!("Cache hit for {}", name);
            return Ok(pkg.clone());
        }

        let url = format!("{}/{}", self.registry_url, encode_package_name(name));
        debug!("Fetching package metadata from {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(SnpmError::PackageNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(SnpmError::Registry(format!(
                "Failed to fetch {}: HTTP {}",
                name,
                response.status()
            )));
        }

        let package: RegistryPackage = response.json().await?;

        // Cache the result
        self.cache.insert(name.to_string(), package.clone());

        Ok(package)
    }

    /// Fetch a specific version of a package.
    #[instrument(skip(self))]
    pub async fn get_package_version(&self, name: &str, version: &str) -> Result<PackageVersion> {
        let package = self.get_package(name).await?;

        package
            .versions
            .get(version)
            .cloned()
            .ok_or_else(|| SnpmError::VersionNotFound {
                package: name.to_string(),
                version: version.to_string(),
            })
    }

    /// Get the latest version of a package.
    pub async fn get_latest_version(&self, name: &str) -> Result<String> {
        let package = self.get_package(name).await?;

        package
            .dist_tags
            .get("latest")
            .cloned()
            .ok_or_else(|| SnpmError::PackageNotFound(name.to_string()))
    }

    /// Get the tarball URL for a specific version.
    pub async fn get_tarball_url(&self, name: &str, version: &str) -> Result<String> {
        let pkg_version = self.get_package_version(name, version).await?;
        Ok(pkg_version.dist.tarball)
    }

    /// Download a tarball.
    #[instrument(skip(self))]
    pub async fn download_tarball(&self, url: &str) -> Result<bytes::Bytes> {
        debug!("Downloading tarball from {}", url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(SnpmError::Registry(format!(
                "Failed to download tarball: HTTP {}",
                response.status()
            )));
        }

        Ok(response.bytes().await?)
    }

    /// Search for packages.
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str, limit: usize) -> Result<SearchResults> {
        let url = format!(
            "{}/-/v1/search?text={}&size={}",
            self.registry_url,
            urlencoding::encode(query),
            limit
        );

        debug!("Searching packages: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(SnpmError::Registry(format!(
                "Search failed: HTTP {}",
                response.status()
            )));
        }

        Ok(response.json().await?)
    }

    /// Fetch abbreviated package metadata (smaller response).
    #[instrument(skip(self))]
    pub async fn get_abbreviated_package(&self, name: &str) -> Result<AbbreviatedPackage> {
        let url = format!("{}/{}", self.registry_url, encode_package_name(name));

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.npm.install-v1+json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(SnpmError::PackageNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(SnpmError::Registry(format!(
                "Failed to fetch {}: HTTP {}",
                name,
                response.status()
            )));
        }

        Ok(response.json().await?)
    }

    /// Clear the metadata cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get raw JSON from the registry.
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.registry_url, path);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(SnpmError::Registry(format!(
                "Request failed: HTTP {}",
                response.status()
            )));
        }

        Ok(response.json().await?)
    }
}

/// Encode a package name for use in URLs.
fn encode_package_name(name: &str) -> String {
    if name.starts_with('@') {
        // Scoped package: @scope/name -> @scope%2Fname
        name.replace('/', "%2F")
    } else {
        name.to_string()
    }
}

/// Search results from the registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    /// Search result objects
    pub objects: Vec<SearchResult>,
    /// Total number of results
    pub total: usize,
    /// Search time in milliseconds
    pub time: Option<String>,
}

/// A single search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    /// Package information
    pub package: SearchPackage,
    /// Search score
    pub score: SearchScore,
    /// Search rank
    #[serde(rename = "searchScore")]
    pub search_score: Option<f64>,
}

/// Package information in search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchPackage {
    /// Package name
    pub name: String,
    /// Package scope (for scoped packages)
    pub scope: Option<String>,
    /// Latest version
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Publication date
    pub date: Option<String>,
    /// Author
    pub author: Option<SearchAuthor>,
    /// Publisher
    pub publisher: Option<SearchPublisher>,
    /// Maintainers
    #[serde(default)]
    pub maintainers: Vec<SearchMaintainer>,
    /// Links
    pub links: Option<SearchLinks>,
}

/// Author in search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchAuthor {
    /// Author name
    pub name: Option<String>,
    /// Author email
    pub email: Option<String>,
}

/// Publisher in search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchPublisher {
    /// Publisher username
    pub username: String,
    /// Publisher email
    pub email: Option<String>,
}

/// Maintainer in search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchMaintainer {
    /// Maintainer username
    pub username: String,
    /// Maintainer email
    pub email: Option<String>,
}

/// Links in search results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchLinks {
    /// NPM page
    pub npm: Option<String>,
    /// Homepage
    pub homepage: Option<String>,
    /// Repository
    pub repository: Option<String>,
    /// Bug tracker
    pub bugs: Option<String>,
}

/// Search score.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchScore {
    /// Final score
    pub r#final: f64,
    /// Detail scores
    pub detail: SearchScoreDetail,
}

/// Detailed search scores.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchScoreDetail {
    /// Quality score
    pub quality: f64,
    /// Popularity score
    pub popularity: f64,
    /// Maintenance score
    pub maintenance: f64,
}

/// Abbreviated package metadata (for faster installs).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AbbreviatedPackage {
    /// Package name
    pub name: String,
    /// Distribution tags
    #[serde(rename = "dist-tags")]
    pub dist_tags: std::collections::BTreeMap<String, String>,
    /// Versions (abbreviated)
    pub versions: std::collections::BTreeMap<String, AbbreviatedVersion>,
    /// Last modified time
    pub modified: Option<String>,
}

/// Abbreviated version information.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbbreviatedVersion {
    /// Package name
    pub name: String,
    /// Version
    pub version: String,
    /// Dependencies
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, String>,
    /// Optional dependencies
    #[serde(default)]
    pub optional_dependencies: std::collections::BTreeMap<String, String>,
    /// Peer dependencies
    #[serde(default)]
    pub peer_dependencies: std::collections::BTreeMap<String, String>,
    /// Distribution info
    pub dist: crate::package::PackageDist,
    /// Engines
    #[serde(default)]
    pub engines: std::collections::BTreeMap<String, String>,
    /// Whether it has install scripts
    #[serde(default)]
    pub has_install_script: bool,
    /// Deprecated message
    pub deprecated: Option<String>,
    /// Binary executables
    pub bin: Option<crate::package::BinField>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encode_package_name() {
        assert_eq!(encode_package_name("lodash"), "lodash");
        assert_eq!(encode_package_name("@types/node"), "@types%2Fnode");
        assert_eq!(encode_package_name("@babel/core"), "@babel%2Fcore");
    }
}
