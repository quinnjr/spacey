//! Info command implementation.

use owo_colors::OwoColorize;

use crate::cli::{Cli, InfoArgs};
use crate::commands::CommandContext;
use crate::error::Result;

/// Run the info command.
pub async fn run(args: &InfoArgs, cli: &Cli) -> Result<()> {
    let ctx = CommandContext::new(cli)?;
    let package = ctx.registry.get_package(&args.package).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&package)?);
        return Ok(());
    }

    println!();
    println!(
        "{}",
        format!(
            "{}@{}",
            package.name,
            package
                .dist_tags
                .get("latest")
                .unwrap_or(&"unknown".to_string())
        )
        .cyan()
        .bold()
    );

    if let Some(ref desc) = package.description {
        println!("{}", desc);
    }
    println!();

    if let Some(ref license) = package.license {
        println!("{}: {}", "license".dimmed(), license);
    }

    if let Some(ref homepage) = package.homepage {
        println!("{}: {}", "homepage".dimmed(), homepage);
    }

    println!();
    println!("{}", "dist-tags:".yellow());
    for (tag, version) in &package.dist_tags {
        println!("  {}: {}", tag, version.green());
    }

    println!();
    println!("{}: {}", "versions".yellow(), package.versions.len());

    Ok(())
}
