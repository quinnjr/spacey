//! Link command implementation.

use crate::cli::{Cli, LinkArgs};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &LinkArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Link command not yet implemented".yellow());
    Ok(())
}
