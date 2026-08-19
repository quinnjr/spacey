//! spacey-npm (snpm) - A fast, async, multithreaded NPM-compatible package manager
//!
//! This is the main entry point for the snpm binary.

use clap::Parser;
use owo_colors::OwoColorize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod cli;
mod commands;
mod config;
mod downloader;
mod error;
mod installer;
mod integrity;
mod lockfile;
mod package;
mod peer_deps;
mod registry;
mod resolver;
mod store;
mod toml_lock;

use cli::{Cli, Commands};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Print banner for verbose mode
    if cli.verbose {
        print_banner();
    }

    // Execute the command
    match &cli.command {
        Some(Commands::Install(args)) => commands::install::run(args, &cli).await,
        Some(Commands::Add(args)) => commands::add::run(args, &cli).await,
        Some(Commands::Remove(args)) => commands::remove::run(args, &cli).await,
        Some(Commands::Update(args)) => commands::update::run(args, &cli).await,
        Some(Commands::Init(args)) => commands::init::run(args, &cli).await,
        Some(Commands::Run(args)) => commands::run::run(args, &cli).await,
        Some(Commands::Test(args)) => commands::test::run(args, &cli).await,
        Some(Commands::Publish(args)) => commands::publish::run(args, &cli).await,
        Some(Commands::Pack(args)) => commands::pack::run(args, &cli).await,
        Some(Commands::Link(args)) => commands::link::run(args, &cli).await,
        Some(Commands::Unlink(args)) => commands::unlink::run(args, &cli).await,
        Some(Commands::List(args)) => commands::list::run(args, &cli).await,
        Some(Commands::Outdated(args)) => commands::outdated::run(args, &cli).await,
        Some(Commands::Search(args)) => commands::search::run(args, &cli).await,
        Some(Commands::Info(args)) => commands::info::run(args, &cli).await,
        Some(Commands::Cache(args)) => commands::cache::run(args, &cli).await,
        Some(Commands::Config(args)) => commands::config::run(args, &cli).await,
        Some(Commands::Audit(args)) => commands::audit::run(args, &cli).await,
        Some(Commands::Exec(args)) => commands::exec::run(args, &cli).await,
        Some(Commands::Version(args)) => commands::version::run(args, &cli).await,
        Some(Commands::Ci(args)) => commands::ci::run(args, &cli).await,
        Some(Commands::Store(args)) => commands::store::run(args, &cli).await,
        None => {
            // Default: run install if package.json exists, otherwise show help
            if std::path::Path::new("package.json").exists() {
                commands::install::run(&cli::InstallArgs::default(), &cli).await
            } else {
                println!("{}", "Usage: snpm <command> [options]".yellow());
                println!();
                println!("Run {} for more information", "snpm --help".cyan());
                Ok(())
            }
        }
    }
}

fn print_banner() {
    println!(
        r#"
{}
  _____ _   _ ____  __  __
 / ____| \ | |  _ \|  \/  |
| (___ |  \| | |_) | \  / |
 \___ \| . ` |  __/| |\/| |
 ____) | |\  | |   | |  | |
|_____/|_| \_|_|   |_|  |_|

{} {} - Fast, async, multithreaded package manager
"#,
        "".bright_cyan(),
        "spacey-npm".bright_cyan().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}
