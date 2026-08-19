//! Unlink command implementation.

use crate::cli::{Cli, UnlinkArgs};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &UnlinkArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Unlink command not yet implemented".yellow());
    Ok(())
}
