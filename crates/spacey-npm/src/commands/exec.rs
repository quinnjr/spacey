//! Exec command implementation.

use crate::cli::{Cli, ExecArgs};
use crate::error::{Result, SnpmError};
use std::process::Command;

pub async fn run(args: &ExecArgs, _cli: &Cli) -> Result<()> {
    let bin_path = std::path::PathBuf::from("node_modules/.bin");
    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = if cfg!(windows) {
        format!("{};{}", bin_path.display(), current_path)
    } else {
        format!("{}:{}", bin_path.display(), current_path)
    };

    let status = Command::new(&args.command)
        .args(&args.args)
        .env("PATH", new_path)
        .status()?;

    if !status.success() {
        return Err(SnpmError::ScriptFailed {
            script: args.command.clone(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}
