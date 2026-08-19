//! Remove command implementation.

use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::cli::{Cli, RemoveArgs};
use crate::error::Result;
use crate::package::PackageJson;

/// Run the remove command.
pub async fn run(args: &RemoveArgs, cli: &Cli) -> Result<()> {
    let pkg_json_path = PathBuf::from("package.json");

    if !pkg_json_path.exists() {
        println!("{}", "No package.json found.".yellow());
        return Ok(());
    }

    let mut package_json = PackageJson::read(&pkg_json_path)?;

    for name in &args.packages {
        if package_json.remove_dependency(name) {
            if !cli.quiet {
                println!("{} {}", "Removed".red(), name.cyan());
            }

            // Remove from node_modules
            let pkg_path = PathBuf::from("node_modules").join(name);
            if pkg_path.exists() {
                std::fs::remove_dir_all(&pkg_path)?;
            }
        } else {
            println!("{} {} not found in dependencies", "Warning:".yellow(), name);
        }
    }

    if !args.no_save {
        package_json.write(&pkg_json_path)?;
    }

    Ok(())
}
