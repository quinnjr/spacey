//! Cache command implementation.

use crate::cli::{CacheAction, CacheArgs, Cli};
use crate::commands::CommandContext;
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(args: &CacheArgs, cli: &Cli) -> Result<()> {
    let ctx = CommandContext::new(cli)?;

    match &args.action {
        CacheAction::Clean => {
            ctx.cache.clear().await?;
            println!("{}", "Cache cleared".green());
        }
        CacheAction::List => {
            let packages = ctx.cache.list()?;
            for pkg in packages {
                println!(
                    "{} {} ({})",
                    pkg.name.cyan(),
                    pkg.version.dimmed(),
                    format_bytes(pkg.size)
                );
            }
        }
        CacheAction::Path => {
            println!("{}", ctx.cache.cache_dir().display());
        }
        CacheAction::Verify => {
            let result = ctx.cache.verify().await?;
            println!(
                "Valid: {}, Invalid: {}, Missing: {}",
                result.valid, result.invalid, result.missing
            );
        }
        CacheAction::Add { packages } => {
            println!("{}", "Cache add not yet implemented".yellow());
        }
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
