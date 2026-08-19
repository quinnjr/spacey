//! Command implementations for snpm.

pub mod add;
pub mod audit;
pub mod cache;
pub mod ci;
pub mod config;
pub mod exec;
pub mod info;
pub mod init;
pub mod install;
pub mod link;
pub mod list;
pub mod outdated;
pub mod pack;
pub mod publish;
pub mod remove;
pub mod run;
pub mod search;
pub mod store;
pub mod test;
pub mod unlink;
pub mod update;
pub mod version;

use std::path::PathBuf;

use crate::cache::PackageCache;
use crate::cli::Cli;
use crate::config::Config;
use crate::downloader::PackageDownloader;
use crate::error::Result;
use crate::installer::Installer;
use crate::registry::RegistryClient;
use crate::resolver::Resolver;

/// Common context for command execution.
pub struct CommandContext {
    pub config: Config,
    pub registry: RegistryClient,
    pub cache: PackageCache,
}

impl CommandContext {
    /// Create a new command context.
    pub fn new(cli: &Cli) -> Result<Self> {
        let mut config = Config::load()?;

        // Override config with CLI options
        if let Some(ref registry) = cli.registry {
            config.registry = registry.clone();
        }
        if let Some(concurrency) = cli.concurrency {
            config.concurrency = concurrency;
        }
        if cli.offline {
            config.registry = String::new(); // Disable registry
        }

        let registry =
            RegistryClient::new(Some(&config.registry), cli.insecure || !config.strict_ssl)?;

        let cache = PackageCache::new(config.cache.clone())?;

        Ok(Self {
            config,
            registry,
            cache,
        })
    }

    /// Create a package downloader.
    pub fn downloader(&self, show_progress: bool) -> PackageDownloader {
        PackageDownloader::new(
            self.registry.clone(),
            self.cache.clone(),
            self.config.concurrency,
            show_progress,
        )
    }

    /// Create an installer.
    pub fn installer(&self, show_progress: bool) -> Installer {
        Installer::new(
            self.downloader(show_progress),
            PathBuf::from("node_modules"),
            self.config.scripts,
            self.config.concurrency,
        )
    }

    /// Create a resolver.
    pub fn resolver(&self, lockfile: Option<crate::lockfile::PackageLock>) -> Resolver {
        Resolver::new(
            self.registry.clone(),
            lockfile,
            self.config.legacy_peer_deps,
            self.config.strict_peer_deps,
        )
    }
}
