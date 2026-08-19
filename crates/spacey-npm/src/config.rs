//! Configuration management for snpm.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, SnpmError};
use crate::registry::DEFAULT_REGISTRY;

/// Configuration for snpm.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Registry URL
    pub registry: String,

    /// Cache directory
    pub cache: Option<PathBuf>,

    /// Global packages directory
    pub prefix: Option<PathBuf>,

    /// Authentication tokens (registry -> token)
    #[serde(default)]
    pub auth_tokens: BTreeMap<String, String>,

    /// Number of concurrent downloads
    pub concurrency: usize,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Whether to skip SSL verification
    pub strict_ssl: bool,

    /// Proxy URL
    pub proxy: Option<String>,

    /// HTTPS proxy URL
    pub https_proxy: Option<String>,

    /// Whether to save dependencies by default
    pub save: bool,

    /// Whether to save exact versions
    pub save_exact: bool,

    /// Default save type (prod, dev, optional)
    pub save_prefix: String,

    /// Legacy peer dependencies mode
    pub legacy_peer_deps: bool,

    /// Strict peer dependencies
    pub strict_peer_deps: bool,

    /// Package lock enabled
    pub package_lock: bool,

    /// Audit enabled
    pub audit: bool,

    /// Scripts enabled
    pub scripts: bool,

    /// Progress bar enabled
    pub progress: bool,

    /// Log level
    pub loglevel: String,

    /// Custom config values
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: DEFAULT_REGISTRY.to_string(),
            cache: None,
            prefix: None,
            auth_tokens: BTreeMap::new(),
            concurrency: num_cpus::get() * 2,
            timeout: 60,
            strict_ssl: true,
            proxy: None,
            https_proxy: None,
            save: true,
            save_exact: false,
            save_prefix: "^".to_string(),
            legacy_peer_deps: false,
            strict_peer_deps: false,
            package_lock: true,
            audit: true,
            scripts: true,
            progress: true,
            loglevel: "notice".to_string(),
            extra: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from default locations.
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Load from global config file
        if let Some(global_config_path) = global_config_path() {
            if global_config_path.exists() {
                config.merge_from_file(&global_config_path)?;
            }
        }

        // Load from user config file
        if let Some(user_config_path) = user_config_path() {
            if user_config_path.exists() {
                config.merge_from_file(&user_config_path)?;
            }
        }

        // Load from project .npmrc
        let project_npmrc = PathBuf::from(".npmrc");
        if project_npmrc.exists() {
            config.merge_from_file(&project_npmrc)?;
        }

        // Load from environment variables
        config.load_from_env();

        Ok(config)
    }

    /// Merge configuration from a file.
    fn merge_from_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;

        // Parse .npmrc format (key=value pairs)
        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                self.set(key.trim(), value.trim());
            }
        }

        Ok(())
    }

    /// Load configuration from environment variables.
    fn load_from_env(&mut self) {
        // NPM_CONFIG_* environment variables
        for (key, value) in std::env::vars() {
            if let Some(config_key) = key.strip_prefix("NPM_CONFIG_") {
                let config_key = config_key.to_lowercase().replace('_', "-");
                self.set(&config_key, &value);
            }
        }
    }

    /// Set a configuration value.
    pub fn set(&mut self, key: &str, value: &str) {
        match key {
            "registry" => self.registry = value.to_string(),
            "cache" => self.cache = Some(PathBuf::from(value)),
            "prefix" => self.prefix = Some(PathBuf::from(value)),
            "concurrency" => {
                if let Ok(n) = value.parse() {
                    self.concurrency = n;
                }
            }
            "timeout" => {
                if let Ok(n) = value.parse() {
                    self.timeout = n;
                }
            }
            "strict-ssl" => self.strict_ssl = value == "true",
            "proxy" => self.proxy = Some(value.to_string()),
            "https-proxy" => self.https_proxy = Some(value.to_string()),
            "save" => self.save = value == "true",
            "save-exact" => self.save_exact = value == "true",
            "save-prefix" => self.save_prefix = value.to_string(),
            "legacy-peer-deps" => self.legacy_peer_deps = value == "true",
            "strict-peer-deps" => self.strict_peer_deps = value == "true",
            "package-lock" => self.package_lock = value == "true",
            "audit" => self.audit = value == "true",
            "ignore-scripts" => self.scripts = value != "true",
            "progress" => self.progress = value == "true",
            "loglevel" => self.loglevel = value.to_string(),
            _ => {
                // Handle auth tokens
                if key.starts_with("//") && key.ends_with(":_authToken") {
                    let registry = key.trim_start_matches("//").trim_end_matches(":_authToken");
                    self.auth_tokens
                        .insert(registry.to_string(), value.to_string());
                } else {
                    self.extra.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
        }
    }

    /// Get a configuration value.
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "registry" => Some(self.registry.clone()),
            "cache" => self.cache.as_ref().map(|p| p.display().to_string()),
            "prefix" => self.prefix.as_ref().map(|p| p.display().to_string()),
            "concurrency" => Some(self.concurrency.to_string()),
            "timeout" => Some(self.timeout.to_string()),
            "strict-ssl" => Some(self.strict_ssl.to_string()),
            "proxy" => self.proxy.clone(),
            "https-proxy" => self.https_proxy.clone(),
            "save" => Some(self.save.to_string()),
            "save-exact" => Some(self.save_exact.to_string()),
            "save-prefix" => Some(self.save_prefix.clone()),
            "legacy-peer-deps" => Some(self.legacy_peer_deps.to_string()),
            "strict-peer-deps" => Some(self.strict_peer_deps.to_string()),
            "package-lock" => Some(self.package_lock.to_string()),
            "audit" => Some(self.audit.to_string()),
            "ignore-scripts" => Some((!self.scripts).to_string()),
            "progress" => Some(self.progress.to_string()),
            "loglevel" => Some(self.loglevel.clone()),
            _ => self
                .extra
                .get(key)
                .and_then(|v| v.as_str().map(String::from)),
        }
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> PathBuf {
        self.cache.clone().unwrap_or_else(default_cache_dir)
    }

    /// Get the global prefix directory.
    pub fn prefix_dir(&self) -> PathBuf {
        self.prefix.clone().unwrap_or_else(default_prefix_dir)
    }

    /// Get auth token for a registry.
    pub fn get_auth_token(&self, registry: &str) -> Option<&String> {
        // Try exact match first
        if let Some(token) = self.auth_tokens.get(registry) {
            return Some(token);
        }

        // Try without protocol
        let registry_host = registry
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');

        self.auth_tokens.get(registry_host)
    }

    /// Save configuration to user config file.
    pub fn save(&self) -> Result<()> {
        let config_path = user_config_path()
            .ok_or_else(|| SnpmError::Config("Could not determine config path".into()))?;

        // Create parent directory
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write .npmrc format
        let mut content = String::new();
        content.push_str(&format!("registry={}\n", self.registry));

        if let Some(ref cache) = self.cache {
            content.push_str(&format!("cache={}\n", cache.display()));
        }

        content.push_str(&format!("concurrency={}\n", self.concurrency));
        content.push_str(&format!("timeout={}\n", self.timeout));
        content.push_str(&format!("strict-ssl={}\n", self.strict_ssl));

        for (registry, token) in &self.auth_tokens {
            content.push_str(&format!("//{}:_authToken={}\n", registry, token));
        }

        std::fs::write(config_path, content)?;

        Ok(())
    }
}

/// Get the global config path.
fn global_config_path() -> Option<PathBuf> {
    Some(PathBuf::from("/etc/npmrc"))
}

/// Get the user config path.
fn user_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".npmrc"))
}

/// Get the default cache directory.
fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("snpm")
}

/// Get the default prefix directory.
fn default_prefix_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("snpm")
}
