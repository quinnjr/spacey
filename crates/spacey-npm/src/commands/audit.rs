//! Audit command implementation.

use crate::cli::{AuditArgs, Cli};
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(_args: &AuditArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Audit command not yet implemented".yellow());
    Ok(())
}
