//! Run command implementation.

use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::{Cli, RunArgs};
use crate::error::{Result, SnpmError};
use crate::package::PackageJson;

/// Run the run command.
pub async fn run(args: &RunArgs, cli: &Cli) -> Result<()> {
    let pkg_json_path = PathBuf::from("package.json");

    if !pkg_json_path.exists() {
        return Err(SnpmError::PackageJsonNotFound(".".into()));
    }

    let package_json = PackageJson::read(&pkg_json_path)?;

    let script = package_json
        .scripts
        .get(&args.script)
        .ok_or_else(|| SnpmError::ScriptNotFound(args.script.clone()))?;

    if !cli.quiet {
        println!("{} {}", ">".dimmed(), script.cyan());
    }

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", script]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", script]);
        c
    };

    // Add script arguments
    for arg in &args.args {
        cmd.arg(arg);
    }

    // Set up PATH with node_modules/.bin
    let bin_path = PathBuf::from("node_modules/.bin");
    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = if cfg!(windows) {
        format!("{};{}", bin_path.display(), current_path)
    } else {
        format!("{}:{}", bin_path.display(), current_path)
    };
    cmd.env("PATH", new_path);

    let status = cmd.status()?;

    if !status.success() {
        return Err(SnpmError::ScriptFailed {
            script: args.script.clone(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}
