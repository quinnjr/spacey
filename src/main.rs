// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Spacey - A JavaScript engine inspired by SpiderMonkey, written in Rust
//!
//! This is the main entry point for the spacey CLI/REPL.
//!
//! ## Features
//!
//! - Interactive REPL with syntax highlighting and history
//! - Async file execution with tokio
//! - Parallel compilation support

mod repl;

use owo_colors::OwoColorize;
use spacey_spidermonkey::AsyncEngine;
use std::env;
use std::path::Path;
use std::process::ExitCode;

/// Main entry point - uses tokio runtime for async operations.
#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        // No arguments - start REPL
        1 => run_repl(),

        // File argument or flags
        _ => {
            let arg = &args[1];

            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    ExitCode::SUCCESS
                }
                "-v" | "--version" => {
                    print_version();
                    ExitCode::SUCCESS
                }
                "-e" | "--eval" => {
                    if args.len() < 3 {
                        eprintln!(
                            "{}: {} requires an argument",
                            "Error".red().bold(),
                            arg.cyan()
                        );
                        ExitCode::FAILURE
                    } else {
                        run_eval(&args[2]).await
                    }
                }
                _ if arg.starts_with('-') => {
                    eprintln!("{}: unknown option '{}'", "Error".red().bold(), arg.cyan());
                    eprintln!("Use {} for usage information", "--help".cyan());
                    ExitCode::FAILURE
                }
                _ => run_file(arg).await,
            }
        }
    }
}

/// Start the interactive REPL
fn run_repl() -> ExitCode {
    match repl::Repl::new() {
        Ok(mut repl) => {
            if let Err(e) = repl.run() {
                eprintln!("{}: {:?}", "REPL Error".red().bold(), e);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!(
                "{}: Failed to initialize REPL: {:?}",
                "Error".red().bold(),
                e
            );
            ExitCode::FAILURE
        }
    }
}

/// Execute a JavaScript file asynchronously.
async fn run_file(path: &str) -> ExitCode {
    let path = Path::new(path);

    if !path.exists() {
        eprintln!(
            "{}: file not found '{}'",
            "Error".red().bold(),
            path.display().cyan()
        );
        return ExitCode::FAILURE;
    }

    let engine = AsyncEngine::new();

    match engine.eval_file(path).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

/// Evaluate JavaScript code from command line asynchronously.
async fn run_eval(code: &str) -> ExitCode {
    let engine = AsyncEngine::new();

    match engine.eval(code).await {
        Ok(value) => {
            if !value.is_undefined() {
                println!("{}", value);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    let version = env!("CARGO_PKG_VERSION");

    println!(
        "{}",
        r#"
   ____
  / ___| _ __   __ _  ___ ___ _   _
  \___ \| '_ \ / _` |/ __/ _ \ | | |
   ___) | |_) | (_| | (_|  __/ |_| |
  |____/| .__/ \__,_|\___\___|\__, |
        |_|                   |___/
"#
        .bright_cyan()
        .bold()
    );

    println!(
        "  {} v{}\n",
        "Spacey JavaScript Engine".white().bold(),
        version.yellow()
    );

    println!("{}", "USAGE:".white().bold());
    println!("    {} [OPTIONS] [FILE]", "spacey".green());
    println!();

    println!("{}", "OPTIONS:".white().bold());
    println!(
        "    {:20} Print this help message",
        "-h, --help".cyan()
    );
    println!(
        "    {:20} Print version information",
        "-v, --version".cyan()
    );
    println!(
        "    {:20} Evaluate JavaScript code",
        "-e, --eval <CODE>".cyan()
    );
    println!();

    println!("{}", "ARGUMENTS:".white().bold());
    println!(
        "    {:20} JavaScript file to execute",
        "[FILE]".cyan()
    );
    println!();

    println!("{}", "EXAMPLES:".white().bold());
    println!(
        "    {}                       # Start interactive REPL",
        "spacey".green()
    );
    println!(
        "    {} {}           # Execute a file",
        "spacey".green(),
        "script.js".dimmed()
    );
    println!(
        "    {} {} {}  # Evaluate expression",
        "spacey".green(),
        "-e".cyan(),
        "\"1 + 2\"".dimmed()
    );
    println!();
}

fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{} {}", "spacey".bright_cyan().bold(), version.yellow());
}
