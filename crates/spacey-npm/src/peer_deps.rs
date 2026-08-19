//! Peer dependency management.
//!
//! Handles peer dependency resolution, validation, and auto-installation.

use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

use crate::error::{Result, SnpmError};
use crate::resolver::ResolvedPackage;

/// Peer dependency configuration.
#[derive(Debug, Clone)]
pub struct PeerDependencyConfig {
    /// Auto-install missing peer dependencies
    pub auto_install: bool,
    /// Strict mode: fail on any peer dependency issue
    pub strict: bool,
    /// Legacy mode: ignore peer dependencies (npm 6 behavior)
    pub legacy: bool,
    /// Allow missing optional peer dependencies
    pub allow_optional_missing: bool,
}

impl Default for PeerDependencyConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            strict: false,
            legacy: false,
            allow_optional_missing: true,
        }
    }
}

/// A peer dependency requirement.
#[derive(Debug, Clone)]
pub struct PeerRequirement {
    /// Package that requires the peer
    pub requirer: String,
    /// Requirer version
    pub requirer_version: String,
    /// Peer package name
    pub peer_name: String,
    /// Peer version requirement
    pub peer_version_req: String,
    /// Whether this peer is optional
    pub optional: bool,
}

/// Result of peer dependency analysis.
#[derive(Debug, Default)]
pub struct PeerAnalysis {
    /// Satisfied peer dependencies
    pub satisfied: Vec<PeerRequirement>,
    /// Missing peer dependencies that should be installed
    pub missing: Vec<PeerRequirement>,
    /// Conflicting peer dependencies
    pub conflicts: Vec<PeerConflict>,
    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,
}

/// A peer dependency conflict.
#[derive(Debug, Clone)]
pub struct PeerConflict {
    /// Peer package name
    pub peer_name: String,
    /// Installed version (if any)
    pub installed_version: Option<String>,
    /// Conflicting requirements
    pub requirements: Vec<PeerRequirement>,
}

/// Peer dependency manager.
pub struct PeerDependencyManager {
    config: PeerDependencyConfig,
}

impl PeerDependencyManager {
    /// Create a new peer dependency manager.
    pub fn new(config: PeerDependencyConfig) -> Self {
        Self { config }
    }

    /// Analyze peer dependencies for a set of resolved packages.
    pub fn analyze(&self, packages: &[ResolvedPackage]) -> PeerAnalysis {
        if self.config.legacy {
            return PeerAnalysis::default();
        }

        let mut analysis = PeerAnalysis::default();

        // Build map of installed packages
        let installed: HashMap<&str, &ResolvedPackage> =
            packages.iter().map(|p| (p.name.as_str(), p)).collect();

        // Collect all peer requirements
        let mut peer_requirements: HashMap<String, Vec<PeerRequirement>> = HashMap::new();

        for pkg in packages {
            for (peer_name, peer_req) in &pkg.peer_dependencies {
                let optional = pkg.optional || self.is_optional_peer(pkg, peer_name);

                let requirement = PeerRequirement {
                    requirer: pkg.name.clone(),
                    requirer_version: pkg.version.clone(),
                    peer_name: peer_name.clone(),
                    peer_version_req: peer_req.clone(),
                    optional,
                };

                peer_requirements
                    .entry(peer_name.clone())
                    .or_default()
                    .push(requirement);
            }
        }

        // Check each peer dependency
        for (peer_name, requirements) in peer_requirements {
            match installed.get(peer_name.as_str()) {
                Some(installed_pkg) => {
                    // Check if installed version satisfies all requirements
                    let mut satisfied_all = true;
                    let mut unsatisfied = Vec::new();

                    for req in &requirements {
                        if !self.version_satisfies(&installed_pkg.version, &req.peer_version_req) {
                            satisfied_all = false;
                            unsatisfied.push(req.clone());
                        } else {
                            analysis.satisfied.push(req.clone());
                        }
                    }

                    if !satisfied_all {
                        // Version conflict
                        analysis.conflicts.push(PeerConflict {
                            peer_name: peer_name.clone(),
                            installed_version: Some(installed_pkg.version.clone()),
                            requirements: unsatisfied,
                        });
                    }
                }
                None => {
                    // Peer not installed
                    let all_optional = requirements.iter().all(|r| r.optional);

                    if all_optional && self.config.allow_optional_missing {
                        for req in &requirements {
                            analysis.warnings.push(format!(
                                "Optional peer dependency {} required by {} is not installed",
                                peer_name, req.requirer
                            ));
                        }
                    } else {
                        // Missing required peer
                        for req in requirements {
                            if !req.optional || !self.config.allow_optional_missing {
                                analysis.missing.push(req);
                            }
                        }
                    }
                }
            }
        }

        analysis
    }

    /// Check if a version satisfies a version requirement.
    fn version_satisfies(&self, version: &str, requirement: &str) -> bool {
        let version = match Version::parse(version) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let req = match parse_version_req(requirement) {
            Ok(r) => r,
            Err(_) => return false,
        };

        req.matches(&version)
    }

    /// Check if a peer dependency is marked as optional.
    fn is_optional_peer(&self, pkg: &ResolvedPackage, peer_name: &str) -> bool {
        // Check peerDependenciesMeta if available
        // For now, assume not optional unless explicitly marked
        false
    }

    /// Get packages that need to be auto-installed for peer dependencies.
    pub fn get_auto_install_packages(&self, analysis: &PeerAnalysis) -> Vec<(String, String)> {
        if !self.config.auto_install {
            return vec![];
        }

        let mut to_install: HashMap<String, String> = HashMap::new();

        for missing in &analysis.missing {
            // Skip if already have a version to install
            if to_install.contains_key(&missing.peer_name) {
                continue;
            }

            // Find the best version requirement
            // For simplicity, use the first requirement
            to_install.insert(missing.peer_name.clone(), missing.peer_version_req.clone());
        }

        to_install.into_iter().collect()
    }

    /// Validate peer dependencies and return errors if strict mode.
    pub fn validate(&self, analysis: &PeerAnalysis) -> Result<()> {
        if self.config.legacy {
            return Ok(());
        }

        // Log warnings
        for warning in &analysis.warnings {
            warn!("{}", warning);
        }

        // Check for conflicts
        if !analysis.conflicts.is_empty() {
            let mut messages = Vec::new();

            for conflict in &analysis.conflicts {
                let installed = conflict
                    .installed_version
                    .as_deref()
                    .unwrap_or("not installed");

                for req in &conflict.requirements {
                    messages.push(format!(
                        "{} requires {}@{}, but {} is {}",
                        req.requirer,
                        conflict.peer_name,
                        req.peer_version_req,
                        conflict.peer_name,
                        installed
                    ));
                }
            }

            if self.config.strict {
                return Err(SnpmError::PeerConflict(messages.join("\n")));
            } else {
                for msg in &messages {
                    warn!("PEER DEP: {}", msg);
                }
            }
        }

        // Check for missing required peers
        let required_missing: Vec<_> = analysis.missing.iter().filter(|m| !m.optional).collect();

        if !required_missing.is_empty() && !self.config.auto_install {
            let mut messages = Vec::new();

            for missing in &required_missing {
                messages.push(format!(
                    "{} requires peer dependency {}@{}",
                    missing.requirer, missing.peer_name, missing.peer_version_req
                ));
            }

            if self.config.strict {
                return Err(SnpmError::PeerConflict(messages.join("\n")));
            } else {
                for msg in &messages {
                    warn!("MISSING PEER: {}", msg);
                }
            }
        }

        Ok(())
    }

    /// Resolve the best version for conflicting peer requirements.
    pub fn resolve_conflict(&self, conflict: &PeerConflict) -> Option<String> {
        if conflict.requirements.is_empty() {
            return None;
        }

        // Try to find a version that satisfies all requirements
        // This is a simplified approach - a full implementation would
        // query the registry for available versions

        // For now, return the most restrictive requirement
        let mut requirements: Vec<_> = conflict
            .requirements
            .iter()
            .filter_map(|r| parse_version_req(&r.peer_version_req).ok())
            .collect();

        if requirements.is_empty() {
            return None;
        }

        // Return the original requirement string of the first one
        // A proper implementation would find the intersection
        Some(conflict.requirements[0].peer_version_req.clone())
    }
}

/// Parse a version requirement string.
fn parse_version_req(version_req: &str) -> Result<VersionReq> {
    let version_req = version_req.trim();

    // Handle exact versions without prefix
    if version_req
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_digit())
    {
        if Version::parse(version_req).is_ok() {
            return VersionReq::parse(&format!("={}", version_req))
                .map_err(|e| SnpmError::Semver(e));
        }
    }

    // Handle x.x.x ranges
    let req = version_req.replace(".x", ".*").replace(".X", ".*");

    VersionReq::parse(&req).map_err(|e| SnpmError::Semver(e))
}

/// Peer dependency deduplication strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerDedupeStrategy {
    /// Hoist peers to the highest possible level
    Hoist,
    /// Keep peers nested where they're used
    Nested,
    /// Automatically choose based on conflicts
    Auto,
}

impl Default for PeerDedupeStrategy {
    fn default() -> Self {
        Self::Auto
    }
}

/// Calculate the optimal peer dependency layout.
pub fn calculate_peer_layout(
    packages: &[ResolvedPackage],
    strategy: PeerDedupeStrategy,
) -> HashMap<String, Vec<String>> {
    let mut layout: HashMap<String, Vec<String>> = HashMap::new();

    // Group packages by their peer dependencies
    let mut peer_users: HashMap<String, Vec<&ResolvedPackage>> = HashMap::new();

    for pkg in packages {
        for peer_name in pkg.peer_dependencies.keys() {
            peer_users.entry(peer_name.clone()).or_default().push(pkg);
        }
    }

    match strategy {
        PeerDedupeStrategy::Hoist => {
            // Hoist all peers to root level
            for peer_name in peer_users.keys() {
                layout.insert(peer_name.clone(), vec!["".to_string()]);
            }
        }
        PeerDedupeStrategy::Nested => {
            // Keep peers nested
            for (peer_name, users) in &peer_users {
                let locations: Vec<String> = users.iter().map(|p| p.name.clone()).collect();
                layout.insert(peer_name.clone(), locations);
            }
        }
        PeerDedupeStrategy::Auto => {
            // Hoist if all users need same version, nest otherwise
            for (peer_name, users) in &peer_users {
                let requirements: HashSet<_> = users
                    .iter()
                    .filter_map(|p| p.peer_dependencies.get(peer_name))
                    .collect();

                if requirements.len() == 1 {
                    // All users need same version, hoist
                    layout.insert(peer_name.clone(), vec!["".to_string()]);
                } else {
                    // Different requirements, keep nested
                    let locations: Vec<String> = users.iter().map(|p| p.name.clone()).collect();
                    layout.insert(peer_name.clone(), locations);
                }
            }
        }
    }

    layout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_satisfies() {
        let manager = PeerDependencyManager::new(PeerDependencyConfig::default());

        assert!(manager.version_satisfies("1.0.0", "^1.0.0"));
        assert!(manager.version_satisfies("1.5.0", "^1.0.0"));
        assert!(!manager.version_satisfies("2.0.0", "^1.0.0"));
        assert!(manager.version_satisfies("1.0.0", ">=1.0.0"));
        assert!(manager.version_satisfies("1.0.0", "*"));
    }
}
