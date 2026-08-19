//! List command implementation.

use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::cli::{Cli, ListArgs};
use crate::error::Result;
use crate::package::PackageJson;

/// Run the list command.
pub async fn run(args: &ListArgs, cli: &Cli) -> Result<()> {
    let pkg_json_path = PathBuf::from("package.json");
    let node_modules = PathBuf::from("node_modules");

    if !node_modules.exists() {
        println!("{}", "node_modules not found.".yellow());
        return Ok(());
    }

    if args.json {
        // TODO: Return JSON format
        println!("{{}}");
        return Ok(());
    }

    // Read root package.json
    let root_pkg = if pkg_json_path.exists() {
        PackageJson::read(&pkg_json_path).ok()
    } else {
        None
    };

    if let Some(pkg) = &root_pkg {
        println!(
            "{}",
            format!(
                "{}@{}",
                pkg.name.as_deref().unwrap_or("(unnamed)"),
                pkg.version.as_deref().unwrap_or("0.0.0")
            )
            .cyan()
            .bold()
        );
    }

    let depth = args.depth.unwrap_or(0);

    // List installed packages
    for entry in std::fs::read_dir(&node_modules)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip hidden directories and .bin
        if name.starts_with('.') {
            continue;
        }

        // Handle scoped packages
        if name.starts_with('@') {
            for scoped_entry in std::fs::read_dir(&path)? {
                let scoped_entry = scoped_entry?;
                let scoped_path = scoped_entry.path();
                if scoped_path.is_dir() {
                    let scoped_name = scoped_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    let full_name = format!("{}/{}", name, scoped_name);
                    print_package(&scoped_path, &full_name, 0, depth, args.long)?;
                }
            }
        } else {
            print_package(&path, name, 0, depth, args.long)?;
        }
    }

    Ok(())
}

fn print_package(
    path: &PathBuf,
    name: &str,
    current_depth: usize,
    max_depth: usize,
    long: bool,
) -> Result<()> {
    let pkg_json_path = path.join("package.json");

    let indent = "  ".repeat(current_depth);
    let prefix = if current_depth == 0 {
        "├── "
    } else {
        "├── "
    };

    if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
        if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
            let version = pkg.version.as_deref().unwrap_or("0.0.0");
            print!("{}{}{}", indent, prefix, name.cyan());
            print!("@{}", version.dimmed());

            if long {
                if let Some(ref desc) = pkg.description {
                    print!(" - {}", desc.dimmed());
                }
            }

            println!();
        }
    } else {
        println!("{}{}{}@{}", indent, prefix, name.cyan(), "?".dimmed());
    }

    Ok(())
}
