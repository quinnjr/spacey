//! Search command implementation.

use owo_colors::OwoColorize;

use crate::cli::{Cli, SearchArgs};
use crate::commands::CommandContext;
use crate::error::Result;

/// Run the search command.
pub async fn run(args: &SearchArgs, cli: &Cli) -> Result<()> {
    let ctx = CommandContext::new(cli)?;

    let query = args.terms.join(" ");
    let results = ctx.registry.search(&query, args.limit).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if results.objects.is_empty() {
        println!("{}", "No packages found.".dimmed());
        return Ok(());
    }

    for result in &results.objects {
        let pkg = &result.package;
        print!("{}", pkg.name.cyan().bold());
        print!(" | {}", pkg.version.dimmed());
        if let Some(ref desc) = pkg.description {
            print!(" | {}", desc);
        }
        println!();

        if args.long {
            if let Some(ref author) = pkg.author {
                if let Some(ref name) = author.name {
                    println!("  Author: {}", name.dimmed());
                }
            }
            if !pkg.keywords.is_empty() {
                println!("  Keywords: {}", pkg.keywords.join(", ").dimmed());
            }
            println!();
        }
    }

    println!();
    println!("{} packages found", results.total.to_string().green());

    Ok(())
}
