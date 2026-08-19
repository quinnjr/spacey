//! CI command implementation.

use crate::cli::{CiArgs, Cli, InstallArgs};
use crate::error::{Result, SnpmError};
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub async fn run(args: &CiArgs, cli: &Cli) -> Result<()> {
    let snpm_toml_path = PathBuf::from("snpm.toml");
    let package_lock_path = PathBuf::from("package-lock.json");

    // Check for lockfile (prefer snpm.toml)
    let has_lockfile = snpm_toml_path.exists() || package_lock_path.exists();

    if !has_lockfile {
        return Err(SnpmError::InvalidLockfile(
            "CI requires a lockfile. Neither snpm.toml nor package-lock.json found.".into(),
        ));
    }

    if !cli.quiet {
        if snpm_toml_path.exists() {
            println!(
                "{} {} {}",
                "CI:".cyan().bold(),
                "Using".dimmed(),
                "snpm.toml".green()
            );
        } else {
            println!(
                "{} {} {}",
                "CI:".cyan().bold(),
                "Using".dimmed(),
                "package-lock.json".yellow()
            );
            println!(
                "  {} Consider using snpm.toml for better reproducibility",
                "Tip:".dimmed()
            );
        }
    }

    // Remove existing node_modules for clean install
    let node_modules = PathBuf::from("node_modules");
    if node_modules.exists() {
        if !cli.quiet {
            println!("{}", "Removing existing node_modules...".dimmed());
        }
        std::fs::remove_dir_all(&node_modules)?;
    }

    // Run install with frozen lockfile mode
    let install_args = InstallArgs {
        ignore_scripts: args.ignore_scripts,
        production: args.production,
        no_package_lock: true, // Don't update lockfile in CI
        frozen_lockfile: true, // Fail if lockfile is out of date
        ..Default::default()
    };

    super::install::run(&install_args, cli).await
}
