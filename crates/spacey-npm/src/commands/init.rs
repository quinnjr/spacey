//! Init command implementation.

use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::cli::{Cli, InitArgs};
use crate::error::Result;
use crate::package::PackageJson;

/// Run the init command.
pub async fn run(args: &InitArgs, cli: &Cli) -> Result<()> {
    let pkg_json_path = PathBuf::from("package.json");

    if pkg_json_path.exists() && !args.force {
        println!(
            "{}",
            "package.json already exists. Use --force to overwrite.".yellow()
        );
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;
    let dir_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-package");

    let name = if let Some(ref scope) = args.scope {
        format!("@{}/{}", scope, dir_name)
    } else {
        dir_name.to_string()
    };

    let package_json = PackageJson {
        name: Some(name.clone()),
        version: Some("1.0.0".to_string()),
        description: Some(String::new()),
        main: Some("index.js".to_string()),
        scripts: [(
            "test".to_string(),
            "echo \"Error: no test specified\" && exit 1".to_string(),
        )]
        .into_iter()
        .collect(),
        keywords: vec![],
        author: None,
        license: Some("ISC".to_string()),
        ..Default::default()
    };

    package_json.write(&pkg_json_path)?;

    if !cli.quiet {
        println!("{} {}", "Created".green(), "package.json".cyan());
        println!();
        println!("{}", serde_json::to_string_pretty(&package_json)?);
    }

    Ok(())
}
