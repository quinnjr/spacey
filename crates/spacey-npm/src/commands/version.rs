//! Version command implementation.

use crate::cli::{Cli, VersionArgs};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &VersionArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Version command not yet implemented".yellow());
    Ok(())
}
