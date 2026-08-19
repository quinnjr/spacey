//! Store command implementation.

use owo_colors::OwoColorize;

use crate::cli::{Cli, StoreAction, StoreArgs};
use crate::error::Result;
use crate::store::PackageStore;

/// Run the store command.
pub async fn run(args: &StoreArgs, cli: &Cli) -> Result<()> {
    let store = PackageStore::new(None)?;

    match &args.action {
        StoreAction::Path => {
            println!("{}", store.store_dir().display());
        }

        StoreAction::Status => {
            let stats = store.stats();

            println!("{}", "Store Status".cyan().bold());
            println!();
            println!("  {}: {}", "Location".dimmed(), store.store_dir().display());
            println!("  {}: {}", "Packages".dimmed(), stats.package_count);
            println!(
                "  {}: {}",
                "Unique packages".dimmed(),
                stats.unique_packages
            );
            println!(
                "  {}: {}",
                "Total size".dimmed(),
                format_bytes(stats.total_size)
            );
            println!(
                "  {}: {}",
                "Space saved (est.)".dimmed(),
                format_bytes(stats.space_saved).green()
            );
        }

        StoreAction::Prune { dry_run } => {
            if *dry_run {
                println!("{}", "Dry run - no changes will be made".yellow());
            }

            // Get all packages currently in use
            // For now, prune all (in real impl, would scan projects)
            let keep: Vec<String> = vec![];

            if *dry_run {
                let stats = store.stats();
                println!(
                    "Would prune {} packages, freeing {}",
                    stats.package_count,
                    format_bytes(stats.total_size)
                );
            } else {
                let result = store.prune(&keep).await?;
                println!(
                    "{} Pruned {} packages, freed {}",
                    "✓".green(),
                    result.removed_count,
                    format_bytes(result.freed_bytes)
                );
            }
        }

        StoreAction::Add { packages } => {
            if packages.is_empty() {
                println!("{}", "No packages specified".yellow());
                return Ok(());
            }

            println!("{}", "Adding packages to store...".dimmed());
            // TODO: Implement package addition to store
            println!("{}", "Store add not yet fully implemented".yellow());
        }

        StoreAction::List { filter } => {
            let packages = store.list_packages();

            let filtered: Vec<_> = if let Some(f) = filter {
                packages.iter().filter(|p| p.name.contains(f)).collect()
            } else {
                packages.iter().collect()
            };

            if filtered.is_empty() {
                println!("{}", "No packages in store".dimmed());
                return Ok(());
            }

            println!("{} packages in store:", filtered.len());
            println!();

            for pkg in filtered {
                println!(
                    "  {}@{} ({})",
                    pkg.name.cyan(),
                    pkg.version.dimmed(),
                    format_bytes(pkg.size).dimmed()
                );
            }
        }
    }

    Ok(())
}

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
