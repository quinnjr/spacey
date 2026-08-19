//! Config command implementation.

use crate::cli::{Cli, ConfigAction, ConfigArgs};
use crate::config::Config;
use crate::error::Result;
use owo_colors::OwoColorize;

pub async fn run(args: &ConfigArgs, _cli: &Cli) -> Result<()> {
    let mut config = Config::load()?;

    match &args.action {
        ConfigAction::Get { key } => {
            if let Some(value) = config.get(key) {
                println!("{}", value);
            } else {
                println!("{}", "undefined".dimmed());
            }
        }
        ConfigAction::Set { key, value } => {
            config.set(key, value);
            config.save()?;
            println!("{} {} = {}", "Set".green(), key.cyan(), value);
        }
        ConfigAction::Delete { key } => {
            config.set(key, "");
            config.save()?;
            println!("{} {}", "Deleted".red(), key.cyan());
        }
        ConfigAction::List => {
            println!("{}: {}", "registry".cyan(), config.registry);
            println!("{}: {}", "cache".cyan(), config.cache_dir().display());
            println!("{}: {}", "concurrency".cyan(), config.concurrency);
        }
        ConfigAction::Edit => {
            println!("{}", "Config edit not yet implemented".yellow());
        }
    }

    Ok(())
}
