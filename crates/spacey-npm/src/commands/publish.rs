//! Publish command implementation.

use crate::cli::{Cli, PublishArgs};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &PublishArgs, cli: &Cli) -> Result<()> {
    println!("{}", "Publish command not yet implemented".yellow());
    Ok(())
}
