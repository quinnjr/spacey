//! Dependency resolution.

use semver::{Version, VersionReq};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::error::{Result, SnpmError};
use crate::lockfile::PackageLock;
use crate::package::{DependencyType, PackageJson, PackageVersion, RegistryPackage};
use crate::registry::RegistryClient;

/// Resolved package information.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// Package name
    pub name: String,
    /// Resolved version
    pub version: String,
    /// Tarball URL
    pub tarball_url: String,
    /// Integrity hash
    pub integrity: Option<String>,
    /// SHA-1 hash (legacy)
    pub shasum: Option<String>,
    /// Dependencies
    pub dependencies: BTreeMap<String, String>,
    /// Peer dependencies
    pub peer_dependencies: BTreeMap<String, String>,
    /// Optional dependencies
    pub optional_dependencies: BTreeMap<String, String>,
    /// Dependency type
    pub dep_type: DependencyType,
    /// Whether it's optional
    pub optional: bool,
    /// Has install scripts
    pub has_install_script: bool,
}

/// Dependency resolver.
pub struct Resolver {
    registry: RegistryClient,
    lockfile: Option<PackageLock>,
    resolved: BTreeMap<String, ResolvedPackage>,
    seen: HashSet<String>,
    legacy_peer_deps: bool,
    strict_peer_deps: bool,
}

impl Resolver {
    /// Create a new resolver.
    pub fn new(
        registry: RegistryClient,
        lockfile: Option<PackageLock>,
        legacy_peer_deps: bool,
        strict_peer_deps: bool,
    ) -> Self {
        Self {
            registry,
            lockfile,
            resolved: BTreeMap::new(),
            seen: HashSet::new(),
            legacy_peer_deps,
            strict_peer_deps,
        }
    }

    /// Resolve all dependencies for a package.json.
    pub async fn resolve(&mut self, package_json: &PackageJson) -> Result<Vec<ResolvedPackage>> {
        info!("Resolving dependencies...");

        // Collect all top-level dependencies
        let mut queue: VecDeque<(String, String, DependencyType, bool)> = VecDeque::new();

        // Production dependencies
        for (name, version_req) in &package_json.dependencies {
            queue.push_back((
                name.clone(),
                version_req.clone(),
                DependencyType::Production,
                false,
            ));
        }

        // Dev dependencies
        for (name, version_req) in &package_json.dev_dependencies {
            queue.push_back((
                name.clone(),
                version_req.clone(),
                DependencyType::Development,
                false,
            ));
        }

        // Optional dependencies
        for (name, version_req) in &package_json.optional_dependencies {
            queue.push_back((
                name.clone(),
                version_req.clone(),
                DependencyType::Optional,
                true,
            ));
        }

        // Process queue
        while let Some((name, version_req, dep_type, optional)) = queue.pop_front() {
            let key = format!("{}@{}", name, version_req);

            if self.seen.contains(&key) {
                continue;
            }
            self.seen.insert(key.clone());

            // Try to resolve from lockfile first
            if let Some(resolved) = self.resolve_from_lockfile(&name, &version_req) {
                debug!("Resolved {} from lockfile: {}", name, resolved.version);

                // Queue transitive dependencies
                for (dep_name, dep_version) in &resolved.dependencies {
                    queue.push_back((
                        dep_name.clone(),
                        dep_version.clone(),
                        DependencyType::Production,
                        false,
                    ));
                }

                self.resolved.insert(name.clone(), resolved);
                continue;
            }

            // Resolve from registry
            match self
                .resolve_from_registry(&name, &version_req, dep_type, optional)
                .await
            {
                Ok(resolved) => {
                    debug!("Resolved {} from registry: {}", name, resolved.version);

                    // Queue transitive dependencies
                    for (dep_name, dep_version) in &resolved.dependencies {
                        queue.push_back((
                            dep_name.clone(),
                            dep_version.clone(),
                            DependencyType::Production,
                            false,
                        ));
                    }

                    // Queue optional dependencies
                    for (dep_name, dep_version) in &resolved.optional_dependencies {
                        queue.push_back((
                            dep_name.clone(),
                            dep_version.clone(),
                            DependencyType::Optional,
                            true,
                        ));
                    }

                    self.resolved.insert(name.clone(), resolved);
                }
                Err(e) => {
                    if optional {
                        warn!("Skipping optional dependency {}: {}", name, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Check peer dependencies
        if !self.legacy_peer_deps {
            self.check_peer_dependencies()?;
        }

        info!("Resolved {} packages", self.resolved.len());
        Ok(self.resolved.values().cloned().collect())
    }

    /// Resolve a specific package.
    pub async fn resolve_package(
        &mut self,
        name: &str,
        version_req: &str,
    ) -> Result<ResolvedPackage> {
        self.resolve_from_registry(name, version_req, DependencyType::Production, false)
            .await
    }

    /// Try to resolve from lockfile.
    fn resolve_from_lockfile(&self, name: &str, version_req: &str) -> Option<ResolvedPackage> {
        let lockfile = self.lockfile.as_ref()?;

        let path = format!("node_modules/{}", name);
        let locked = lockfile.get_package(&path)?;

        // Check if locked version satisfies requirement
        if let Ok(req) = parse_version_req(version_req) {
            if let Ok(version) = Version::parse(&locked.version) {
                if req.matches(&version) {
                    return Some(ResolvedPackage {
                        name: name.to_string(),
                        version: locked.version.clone(),
                        tarball_url: locked.resolved.clone().unwrap_or_default(),
                        integrity: locked.integrity.clone(),
                        shasum: None,
                        dependencies: locked.dependencies.clone(),
                        peer_dependencies: locked.peer_dependencies.clone(),
                        optional_dependencies: locked.optional_dependencies.clone(),
                        dep_type: DependencyType::Production,
                        optional: locked.optional,
                        has_install_script: locked.has_install_script,
                    });
                }
            }
        }

        None
    }

    /// Resolve from registry.
    async fn resolve_from_registry(
        &self,
        name: &str,
        version_req: &str,
        dep_type: DependencyType,
        optional: bool,
    ) -> Result<ResolvedPackage> {
        let package = self.registry.get_package(name).await?;
        let version = self.find_best_version(&package, version_req)?;

        let pkg_version =
            package
                .versions
                .get(&version)
                .ok_or_else(|| SnpmError::VersionNotFound {
                    package: name.to_string(),
                    version: version.clone(),
                })?;

        Ok(ResolvedPackage {
            name: name.to_string(),
            version: version.clone(),
            tarball_url: pkg_version.dist.tarball.clone(),
            integrity: pkg_version.dist.integrity.clone(),
            shasum: pkg_version.dist.shasum.clone(),
            dependencies: pkg_version.dependencies.clone(),
            peer_dependencies: pkg_version.peer_dependencies.clone(),
            optional_dependencies: pkg_version.optional_dependencies.clone(),
            dep_type,
            optional,
            has_install_script: pkg_version.has_install_script,
        })
    }

    /// Find the best version matching a requirement.
    fn find_best_version(&self, package: &RegistryPackage, version_req: &str) -> Result<String> {
        // Handle special version strings
        match version_req {
            "latest" => {
                return package.dist_tags.get("latest").cloned().ok_or_else(|| {
                    SnpmError::VersionNotFound {
                        package: package.name.clone(),
                        version: "latest".into(),
                    }
                });
            }
            v if v.starts_with("npm:") => {
                // Aliased package: npm:package@version
                // For now, just use the version part
                let parts: Vec<&str> = v.splitn(2, '@').collect();
                if parts.len() == 2 {
                    return self.find_best_version(package, parts[1]);
                }
            }
            v if v.starts_with("git") || v.starts_with("http") || v.starts_with("file:") => {
                return Err(SnpmError::ResolutionError(format!(
                    "URL/git dependencies not yet supported: {}",
                    v
                )));
            }
            _ => {}
        }

        let req = parse_version_req(version_req)?;

        // Find all matching versions
        let mut matching: Vec<Version> = package
            .versions
            .keys()
            .filter_map(|v| Version::parse(v).ok())
            .filter(|v| req.matches(v))
            .collect();

        // Sort by version (highest first)
        matching.sort_by(|a, b| b.cmp(a));

        matching
            .first()
            .map(|v| v.to_string())
            .ok_or_else(|| SnpmError::VersionNotFound {
                package: package.name.clone(),
                version: version_req.to_string(),
            })
    }

    /// Check peer dependencies.
    fn check_peer_dependencies(&self) -> Result<()> {
        for (name, resolved) in &self.resolved {
            for (peer_name, peer_req) in &resolved.peer_dependencies {
                if let Some(peer_resolved) = self.resolved.get(peer_name) {
                    // Check if installed version satisfies peer requirement
                    if let Ok(req) = parse_version_req(peer_req) {
                        if let Ok(version) = Version::parse(&peer_resolved.version) {
                            if !req.matches(&version) {
                                let msg = format!(
                                    "{} requires {}@{}, but {} is installed",
                                    name, peer_name, peer_req, peer_resolved.version
                                );
                                if self.strict_peer_deps {
                                    return Err(SnpmError::PeerConflict(msg));
                                } else {
                                    warn!("Peer dependency warning: {}", msg);
                                }
                            }
                        }
                    }
                } else {
                    let msg = format!(
                        "{} requires peer dependency {}@{}",
                        name, peer_name, peer_req
                    );
                    if self.strict_peer_deps {
                        return Err(SnpmError::PeerConflict(msg));
                    } else {
                        warn!("Missing peer dependency: {}", msg);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get resolved packages.
    pub fn resolved_packages(&self) -> &BTreeMap<String, ResolvedPackage> {
        &self.resolved
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
        // Check if it's a valid semver
        if Version::parse(version_req).is_ok() {
            // Exact version, convert to requirement
            return VersionReq::parse(&format!("={}", version_req))
                .map_err(|e| SnpmError::Semver(e));
        }
    }

    // Handle x.x.x ranges
    let req = version_req.replace(".x", ".*").replace(".X", ".*");

    VersionReq::parse(&req).map_err(|e| SnpmError::Semver(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_req() {
        assert!(parse_version_req("^1.0.0").is_ok());
        assert!(parse_version_req("~1.0.0").is_ok());
        assert!(parse_version_req(">=1.0.0").is_ok());
        assert!(parse_version_req("1.0.0").is_ok());
        assert!(parse_version_req("1.x").is_ok());
        assert!(parse_version_req("*").is_ok());
    }
}
