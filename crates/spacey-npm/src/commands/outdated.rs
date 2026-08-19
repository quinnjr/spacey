//! Outdated command implementation.

use crate::cli::{Cli, OutdatedArgs};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &OutdatedArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Checking for outdated packages...".dimmed());
    }
    // TODO: Implement
    println!("{}", "Outdated command not yet implemented".yellow());
    Ok(())
}
