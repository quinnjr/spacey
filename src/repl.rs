// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Interactive REPL (Read-Eval-Print Loop) for Spacey JavaScript Engine.

use owo_colors::OwoColorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Config, Editor, Helper};
use spacey_spidermonkey::{Engine, Value};
use std::borrow::Cow;
use std::path::PathBuf;

/// REPL configuration constants
const HISTORY_FILE: &str = ".spacey_history";
const MAX_HISTORY_SIZE: usize = 1000;

/// REPL commands that can be executed with a dot prefix
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplCommand {
    Help,
    Exit,
    Clear,
    Version,
    Load,
}

impl ReplCommand {
    /// Parse a REPL command from input string
    pub fn parse(input: &str) -> Option<(Self, Option<&str>)> {
        let input = input.trim();
        if !input.starts_with('.') {
            return None;
        }

        let parts: Vec<&str> = input[1..].splitn(2, char::is_whitespace).collect();
        let cmd = parts.first()?.to_lowercase();
        let arg = parts.get(1).copied();

        match cmd.as_str() {
            "help" | "h" | "?" => Some((ReplCommand::Help, arg)),
            "exit" | "quit" | "q" => Some((ReplCommand::Exit, arg)),
            "clear" | "cls" => Some((ReplCommand::Clear, arg)),
            "version" | "v" => Some((ReplCommand::Version, arg)),
            "load" | "l" => Some((ReplCommand::Load, arg)),
            _ => None,
        }
    }

    /// Get all available commands for help/completion
    pub fn all_commands() -> &'static [(&'static str, &'static str)] {
        &[
            (".help", "Show this help message"),
            (".exit", "Exit the REPL"),
            (".clear", "Clear the screen"),
            (".version", "Show version information"),
            (".load <file>", "Load and execute a JavaScript file"),
        ]
    }
}

/// Helper struct for rustyline that provides completion, hints, and validation
#[derive(Default)]
struct SpaceyHelper {
    /// Keywords and built-in identifiers for completion
    keywords: Vec<String>,
}

impl SpaceyHelper {
    fn new() -> Self {
        let keywords = vec![
            // Keywords
            "async",
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "let",
            "new",
            "null",
            "return",
            "static",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "undefined",
            "var",
            "void",
            "while",
            "with",
            "yield",
            // Global objects
            "Array",
            "Boolean",
            "console",
            "Date",
            "Error",
            "Function",
            "JSON",
            "Math",
            "Number",
            "Object",
            "Promise",
            "RegExp",
            "String",
            "Symbol",
            // Common methods
            "console.log",
            "console.error",
            "console.warn",
            // REPL commands
            ".help",
            ".exit",
            ".clear",
            ".version",
            ".load",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self { keywords }
    }
}

impl Completer for SpaceyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);

        let word = &line[start..pos];
        if word.is_empty() {
            return Ok((pos, vec![]));
        }

        let matches: Vec<Pair> = self
            .keywords
            .iter()
            .filter(|kw| kw.starts_with(word))
            .map(|kw| Pair {
                display: kw.clone(),
                replacement: kw[word.len()..].to_string(),
            })
            .collect();

        Ok((pos, matches))
    }
}

impl Hinter for SpaceyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if pos < line.len() {
            return None;
        }

        // Find the start of the current word
        let start = line
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);

        let word = &line[start..];
        if word.len() < 2 {
            return None;
        }

        // Find first matching keyword
        self.keywords
            .iter()
            .find(|kw| kw.starts_with(word) && kw.len() > word.len())
            .map(|kw| kw[word.len()..].to_string().dimmed().to_string())
    }
}

impl Highlighter for SpaceyHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Basic syntax highlighting
        let mut result = String::with_capacity(line.len() * 2);
        let chars = line.chars().peekable();
        let mut current_word = String::new();

        for c in chars {
            if c.is_alphanumeric() || c == '_' {
                current_word.push(c);
            } else {
                if !current_word.is_empty() {
                    result.push_str(&highlight_word(&current_word));
                    current_word.clear();
                }
                // Color operators and punctuation
                let colored = match c {
                    '(' | ')' | '[' | ']' | '{' | '}' => c.to_string().yellow().to_string(),
                    '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '&' | '|' | '^' => {
                        c.to_string().cyan().to_string()
                    }
                    '"' | '\'' | '`' => c.to_string().green().to_string(),
                    '.' if line.starts_with('.') => c.to_string().magenta().to_string(),
                    _ => c.to_string(),
                };
                result.push_str(&colored);
            }
        }

        if !current_word.is_empty() {
            result.push_str(&highlight_word(&current_word));
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

fn highlight_word(word: &str) -> String {
    // JavaScript keywords
    const KEYWORDS: &[&str] = &[
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "let",
        "new",
        "return",
        "static",
        "super",
        "switch",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ];

    const LITERALS: &[&str] = &[
        "true",
        "false",
        "null",
        "undefined",
        "NaN",
        "Infinity",
        "this",
    ];

    const BUILTINS: &[&str] = &[
        "Array", "Boolean", "console", "Date", "Error", "Function", "JSON", "Math", "Number",
        "Object", "Promise", "RegExp", "String", "Symbol", "Map", "Set", "WeakMap", "WeakSet",
    ];

    if KEYWORDS.contains(&word) {
        word.magenta().bold().to_string()
    } else if LITERALS.contains(&word) {
        word.blue().to_string()
    } else if BUILTINS.contains(&word) {
        word.cyan().to_string()
    } else if word.chars().all(|c| c.is_ascii_digit() || c == '.') {
        word.yellow().to_string()
    } else {
        word.to_string()
    }
}

impl Validator for SpaceyHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // Check for balanced brackets/braces/parentheses
        if !is_balanced(input) {
            return Ok(ValidationResult::Incomplete);
        }

        // Check if line ends with an operator that expects more input
        let trimmed = input.trim();
        if trimmed.ends_with('\\')
            || trimmed.ends_with('+')
            || trimmed.ends_with('-')
            || trimmed.ends_with('*')
            || trimmed.ends_with('/')
            || trimmed.ends_with('=')
            || trimmed.ends_with(',')
            || trimmed.ends_with('{')
            || trimmed.ends_with('(')
            || trimmed.ends_with('[')
        {
            return Ok(ValidationResult::Incomplete);
        }

        Ok(ValidationResult::Valid(None))
    }
}

/// Check if brackets, braces, and parentheses are balanced
fn is_balanced(input: &str) -> bool {
    let mut stack = Vec::new();
    let mut in_string = None;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string.is_some() {
            escape_next = true;
            continue;
        }

        match in_string {
            Some(quote) if c == quote => in_string = None,
            Some(_) => {}
            None => match c {
                '"' | '\'' | '`' => in_string = Some(c),
                '(' => stack.push(')'),
                '[' => stack.push(']'),
                '{' => stack.push('}'),
                ')' | ']' | '}' => {
                    if stack.pop() != Some(c) {
                        return true; // Unbalanced but we should let the parser handle the error
                    }
                }
                _ => {}
            },
        }
    }

    stack.is_empty() && in_string.is_none()
}

impl Helper for SpaceyHelper {}

/// The interactive REPL for the Spacey JavaScript engine
pub struct Repl {
    engine: Engine,
    editor: Editor<SpaceyHelper, DefaultHistory>,
    history_path: PathBuf,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> rustyline::Result<Self> {
        let config = Config::builder()
            .history_ignore_dups(true)?
            .history_ignore_space(true)
            .max_history_size(MAX_HISTORY_SIZE)?
            .auto_add_history(true)
            .build();

        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(SpaceyHelper::new()));

        // Determine history file path
        let history_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("spacey")
            .join(HISTORY_FILE);

        // Create parent directory if it doesn't exist
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Load history
        let _ = editor.load_history(&history_path);

        Ok(Self {
            engine: Engine::new(),
            editor,
            history_path,
        })
    }

    /// Run the REPL main loop
    pub fn run(&mut self) -> rustyline::Result<()> {
        self.print_banner();

        loop {
            let prompt = self.format_prompt(false);

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let trimmed = line.trim();

                    if trimmed.is_empty() {
                        continue;
                    }

                    // Check for REPL commands
                    if let Some((cmd, arg)) = ReplCommand::parse(trimmed) {
                        match self.execute_command(cmd, arg) {
                            CommandResult::Continue => continue,
                            CommandResult::Exit => break,
                        }
                    }

                    // Evaluate JavaScript
                    self.eval_and_print(trimmed);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C".dimmed());
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "^D".dimmed());
                    break;
                }
                Err(err) => {
                    eprintln!("{}: {:?}", "Error".red().bold(), err);
                    break;
                }
            }
        }

        // Save history
        let _ = self.editor.save_history(&self.history_path);

        self.print_goodbye();
        Ok(())
    }

    fn print_banner(&self) {
        let version = env!("CARGO_PKG_VERSION");
        println!();
        println!(
            "{}",
            r#"   ____                          "#.bright_cyan().bold()
        );
        println!(
            "{}",
            r#"  / ___| _ __   __ _  ___ ___ _   _ "#.bright_cyan().bold()
        );
        println!(
            "{}",
            r#"  \___ \| '_ \ / _` |/ __/ _ \ | | |"#.bright_cyan().bold()
        );
        println!(
            "{}",
            r#"   ___) | |_) | (_| | (_|  __/ |_| |"#.bright_cyan().bold()
        );
        println!(
            "{}",
            r#"  |____/| .__/ \__,_|\___\___|\__, |"#.bright_cyan().bold()
        );
        println!(
            "{}",
            r#"        |_|                   |___/ "#.bright_cyan().bold()
        );
        println!();
        println!(
            "  {} {} {}",
            "Spacey JavaScript Engine".white().bold(),
            "v".dimmed(),
            version.bright_yellow()
        );
        println!(
            "  {}",
            "A JavaScript engine for the Rust-powered web".dimmed()
        );
        println!();
        println!(
            "  {} {} {}",
            "Type".dimmed(),
            ".help".cyan(),
            "for available commands".dimmed()
        );
        println!();
    }

    fn print_goodbye(&self) {
        println!();
        println!("{}", "Goodbye! 👋".bright_cyan());
        println!();
    }

    fn format_prompt(&self, multiline: bool) -> String {
        if multiline {
            format!("{} ", "...".dimmed())
        } else {
            format!("{} ", "spacey>".bright_green().bold())
        }
    }

    fn execute_command(&mut self, cmd: ReplCommand, arg: Option<&str>) -> CommandResult {
        match cmd {
            ReplCommand::Help => {
                self.print_help();
                CommandResult::Continue
            }
            ReplCommand::Exit => CommandResult::Exit,
            ReplCommand::Clear => {
                print!("\x1B[2J\x1B[H");
                CommandResult::Continue
            }
            ReplCommand::Version => {
                self.print_version();
                CommandResult::Continue
            }
            ReplCommand::Load => {
                if let Some(path) = arg {
                    self.load_file(path);
                } else {
                    eprintln!(
                        "{}: {} {}",
                        "Error".red().bold(),
                        ".load".cyan(),
                        "requires a file path".dimmed()
                    );
                }
                CommandResult::Continue
            }
        }
    }

    fn print_help(&self) {
        println!();
        println!("{}", "REPL Commands:".white().bold());
        println!();

        for (cmd, desc) in ReplCommand::all_commands() {
            println!("  {:16} {}", cmd.cyan(), desc.dimmed());
        }

        println!();
        println!("{}", "Keyboard Shortcuts:".white().bold());
        println!();
        println!(
            "  {:16} {}",
            "Ctrl+C".yellow(),
            "Cancel current input".dimmed()
        );
        println!("  {:16} {}", "Ctrl+D".yellow(), "Exit REPL".dimmed());
        println!("  {:16} {}", "Ctrl+L".yellow(), "Clear screen".dimmed());
        println!("  {:16} {}", "Tab".yellow(), "Autocomplete".dimmed());
        println!("  {:16} {}", "↑/↓".yellow(), "Navigate history".dimmed());
        println!();
    }

    fn print_version(&self) {
        let version = env!("CARGO_PKG_VERSION");
        println!();
        println!("{}: {}", "Spacey".bright_cyan().bold(), version.yellow());
        println!("{}: {}", "Rust".dimmed(), env!("CARGO_PKG_RUST_VERSION"));
        println!();
    }

    fn load_file(&mut self, path: &str) {
        let path = std::path::Path::new(path.trim());

        match self.engine.eval_file(path) {
            Ok(value) => {
                println!("{}", format_value(&value));
            }
            Err(e) => {
                print_error(&e);
            }
        }
    }

    fn eval_and_print(&mut self, input: &str) {
        match self.engine.eval(input) {
            Ok(value) => {
                println!("{}", format_value(&value));
            }
            Err(e) => {
                print_error(&e);
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to initialize REPL")
    }
}

/// Result of executing a REPL command
enum CommandResult {
    Continue,
    Exit,
}

/// Format a JavaScript value for display with syntax coloring
fn format_value(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".blue().dimmed().to_string(),
        Value::Null => "null".blue().to_string(),
        Value::Boolean(b) => b.to_string().yellow().to_string(),
        Value::Number(n) => {
            if n.is_nan() {
                "NaN".yellow().to_string()
            } else if n.is_infinite() {
                if *n > 0.0 {
                    "Infinity".yellow().to_string()
                } else {
                    "-Infinity".yellow().to_string()
                }
            } else {
                n.to_string().yellow().to_string()
            }
        }
        Value::String(s) => format!("'{}'", s).green().to_string(),
        Value::Symbol(id) => format!("Symbol({})", id).magenta().to_string(),
        Value::BigInt(n) => format!("{}n", n).yellow().to_string(),
        Value::Object(_) | Value::NativeObject(_) => "[object Object]".cyan().to_string(),
        Value::Function(_) => "[Function]".magenta().to_string(),
    }
}

/// Print a formatted error message
fn print_error(error: &spacey_spidermonkey::Error) {
    let error_str = error.to_string();

    // Split error type from message
    if let Some(colon_pos) = error_str.find(':') {
        let (error_type, message) = error_str.split_at(colon_pos);
        eprintln!("{}{}", error_type.red().bold(), message);
    } else {
        eprintln!("{}", error_str.red());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_command_parse() {
        assert!(matches!(
            ReplCommand::parse(".help"),
            Some((ReplCommand::Help, None))
        ));
        assert!(matches!(
            ReplCommand::parse(".exit"),
            Some((ReplCommand::Exit, None))
        ));
        assert!(matches!(
            ReplCommand::parse(".load test.js"),
            Some((ReplCommand::Load, Some("test.js")))
        ));
        assert!(ReplCommand::parse("not a command").is_none());
    }

    #[test]
    fn test_is_balanced() {
        assert!(is_balanced("(1 + 2)"));
        assert!(is_balanced("{ a: 1 }"));
        assert!(is_balanced("function() { return 1; }"));
        assert!(!is_balanced("(1 + 2"));
        assert!(!is_balanced("{ a: 1"));
        assert!(is_balanced("'string with (unbalanced'"));
    }
}
