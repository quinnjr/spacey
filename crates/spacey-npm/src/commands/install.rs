//! Install command implementation.

use std::path::PathBuf;
use std::time::Instant;

use owo_colors::OwoColorize;
use tracing::info;

use crate::cli::{Cli, InstallArgs};
use crate::commands::CommandContext;
use crate::error::Result;
use crate::installer::update_lockfile;
use crate::lockfile::PackageLock;
use crate::package::PackageJson;
use crate::toml_lock::SnpmToml;

/// Run the install command.
pub async fn run(args: &InstallArgs, cli: &Cli) -> Result<()> {
    let start = Instant::now();
    let ctx = CommandContext::new(cli)?;

    // Check for package.json
    let pkg_json_path = PathBuf::from("package.json");
    if !pkg_json_path.exists() && args.packages.is_empty() {
        println!(
            "{}",
            "No package.json found. Run 'snpm init' first.".yellow()
        );
        return Ok(());
    }

    // Load package.json
    let mut package_json = if pkg_json_path.exists() {
        PackageJson::read(&pkg_json_path)?
    } else {
        PackageJson::default()
    };

    // Load or create lockfile
    let lockfile_path = PathBuf::from("package-lock.json");
    let snpm_toml_path = PathBuf::from("snpm.toml");

    // For frozen lockfile mode, require a lockfile to exist
    if args.frozen_lockfile && !snpm_toml_path.exists() && !lockfile_path.exists() {
        return Err(crate::error::SnpmError::InvalidLockfile(
            "Frozen lockfile mode requires snpm.toml or package-lock.json to exist".into(),
        ));
    }

    // Prefer snpm.toml if it exists (primary lockfile), otherwise use package-lock.json
    let lockfile = if snpm_toml_path.exists() && !args.no_package_lock {
        // Read snpm.toml and convert to PackageLock for resolver
        if !cli.quiet {
            println!("  {} {}", "Reading".dimmed(), "snpm.toml".cyan());
        }
        let toml_lock = SnpmToml::read(&snpm_toml_path)?;
        Some(crate::toml_lock::convert_to_package_lock(&toml_lock))
    } else if lockfile_path.exists() && !args.no_package_lock {
        if !cli.quiet {
            println!("  {} {}", "Reading".dimmed(), "package-lock.json".cyan());
        }
        Some(PackageLock::read(&lockfile_path)?)
    } else {
        None
    };

    // If specific packages are provided, add them to package.json
    if !args.packages.is_empty() {
        for pkg_spec in &args.packages {
            let (name, version) = parse_package_spec(pkg_spec);

            let dep_type = if args.save_dev {
                crate::package::DependencyType::Development
            } else if args.save_optional {
                crate::package::DependencyType::Optional
            } else if args.save_peer {
                crate::package::DependencyType::Peer
            } else {
                crate::package::DependencyType::Production
            };

            // Resolve version if not specified
            let resolved_version = if version == "latest" || version.is_empty() {
                let latest = ctx.registry.get_latest_version(&name).await?;
                if args.save_exact {
                    latest
                } else {
                    format!("^{}", latest)
                }
            } else {
                version.to_string()
            };

            if !cli.quiet {
                println!(
                    "{} {}@{}",
                    "Adding".green(),
                    name.cyan(),
                    resolved_version.dimmed()
                );
            }

            if !args.no_save {
                package_json.add_dependency(&name, &resolved_version, dep_type);
            }
        }

        // Save updated package.json
        if !args.no_save {
            package_json.write(&pkg_json_path)?;
        }
    }

    // Create resolver
    let mut resolver = ctx.resolver(lockfile.clone());

    // Resolve dependencies
    if !cli.quiet {
        println!("{}", "Resolving dependencies...".dimmed());
    }

    let resolved = if args.production {
        // Only production dependencies
        let mut prod_pkg = package_json.clone();
        prod_pkg.dev_dependencies.clear();
        resolver.resolve(&prod_pkg).await?
    } else {
        resolver.resolve(&package_json).await?
    };

    if resolved.is_empty() {
        if !cli.quiet {
            println!("{}", "No packages to install.".dimmed());
        }
        return Ok(());
    }

    // Install packages
    let installer = ctx.installer(!cli.quiet && ctx.config.progress);
    let result = installer.install(&resolved).await?;

    // Update lockfiles (write BOTH snpm.toml and package-lock.json)
    if args.package_lock && !args.no_package_lock {
        // Write snpm.toml (primary lockfile)
        let toml_lock = SnpmToml::from_resolved(
            package_json.name.clone().unwrap_or_default(),
            package_json.version.clone().unwrap_or_default(),
            &resolved,
        );
        toml_lock.write(&snpm_toml_path)?;

        if !cli.quiet {
            println!("  {} {}", "Wrote".dimmed(), "snpm.toml".cyan());
        }

        // Write package-lock.json (for npm compatibility)
        let mut lock = lockfile.unwrap_or_else(|| {
            PackageLock::new(
                package_json.name.clone().unwrap_or_default(),
                package_json.version.clone().unwrap_or_default(),
            )
        });

        // Add root package entry
        lock.add_package(
            "",
            crate::lockfile::LockPackage {
                version: package_json.version.clone().unwrap_or_default(),
                dependencies: package_json.dependencies.clone(),
                dev_dependencies: package_json.dev_dependencies.clone(),
                peer_dependencies: package_json.peer_dependencies.clone(),
                optional_dependencies: package_json.optional_dependencies.clone(),
                ..Default::default()
            },
        );

        update_lockfile(&mut lock, &resolved);
        lock.write(&lockfile_path)?;

        if !cli.quiet {
            println!("  {} {}", "Wrote".dimmed(), "package-lock.json".cyan());
        }
    }

    // Print summary
    let elapsed = start.elapsed();
    if !cli.quiet {
        println!();
        println!(
            "{} {} packages in {:.2}s",
            "✓".green().bold(),
            result.packages_installed,
            elapsed.as_secs_f64()
        );

        if result.packages_from_cache > 0 {
            println!(
                "  {} packages from cache",
                result.packages_from_cache.to_string().dimmed()
            );
        }

        if result.bytes_downloaded > 0 {
            println!(
                "  {} downloaded",
                format_bytes(result.bytes_downloaded).dimmed()
            );
        }
    }

    Ok(())
}

/// Parse a package specifier (name@version).
fn parse_package_spec(spec: &str) -> (&str, &str) {
    // Handle scoped packages: @scope/name@version
    if spec.starts_with('@') {
        if let Some(at_pos) = spec[1..].find('@') {
            let at_pos = at_pos + 1;
            return (&spec[..at_pos], &spec[at_pos + 1..]);
        }
        return (spec, "latest");
    }

    // Regular package: name@version
    if let Some(at_pos) = spec.find('@') {
        return (&spec[..at_pos], &spec[at_pos + 1..]);
    }

    (spec, "latest")
}

/// Format bytes as human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        assert_eq!(parse_package_spec("lodash"), ("lodash", "latest"));
        assert_eq!(parse_package_spec("lodash@4.17.21"), ("lodash", "4.17.21"));
        assert_eq!(parse_package_spec("@types/node"), ("@types/node", "latest"));
        assert_eq!(
            parse_package_spec("@types/node@18.0.0"),
            ("@types/node", "18.0.0")
        );
    }
}
