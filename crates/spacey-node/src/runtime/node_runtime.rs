// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Main Node.js runtime implementation

use crate::error::{NodeError, Result};
use crate::globals;
use crate::module_system::{EsmLoader, ModuleLoader, ModuleType};
use crate::modules;
use crate::runtime::event_loop::{Callback, EventLoop};
use owo_colors::OwoColorize;
use parking_lot::RwLock;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use spacey_spidermonkey::{Engine, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// The main Node.js-compatible runtime
pub struct NodeRuntime {
    /// The JavaScript engine
    engine: Engine,
    /// Event loop for async operations
    event_loop: Arc<EventLoop>,
    /// CommonJS module loader and cache
    module_loader: Arc<RwLock<ModuleLoader>>,
    /// ESM module loader
    esm_loader: Arc<EsmLoader>,
    /// Process arguments
    args: Vec<String>,
    /// Current working directory
    cwd: PathBuf,
    /// Exit code (set by process.exit())
    exit_code: Arc<RwLock<Option<i32>>>,
    /// Registered native modules
    native_modules: HashMap<String, Value>,
}

impl NodeRuntime {
    /// Create a new Node.js runtime
    pub fn new(args: Vec<String>) -> Self {
        let engine = Engine::new();
        let event_loop = Arc::new(EventLoop::new());
        let module_loader = Arc::new(RwLock::new(ModuleLoader::new()));
        let esm_loader = Arc::new(EsmLoader::new());
        let exit_code = Arc::new(RwLock::new(None));
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Register globals
        let globals_value =
            globals::create_globals(&args, &cwd, Arc::clone(&event_loop), Arc::clone(&exit_code));

        // Inject globals into engine
        if let Value::Object(_) = &globals_value {
            // The engine should have these globals available
            // We'll register them through the builtins system
        }

        // Register native modules
        let native_modules = modules::create_native_modules();

        Self {
            engine,
            event_loop,
            module_loader,
            esm_loader,
            args,
            cwd,
            exit_code,
            native_modules,
        }
    }

    /// Evaluate JavaScript code
    pub async fn eval(&mut self, code: &str) -> Result<Value> {
        // Check for process.exit() before running
        {
            let exit = self.exit_code.read();
            if let Some(code) = *exit {
                return Err(NodeError::Process(format!(
                    "Process exited with code {}",
                    code
                )));
            }
        }

        // Evaluate the code
        let result = self.engine.eval(code)?;

        // Run the event loop until no more pending work
        self.run_event_loop().await?;

        Ok(result)
    }

    /// Evaluate TypeScript code
    ///
    /// TypeScript syntax (type annotations, interfaces, etc.) is parsed and
    /// stripped at parse time. No separate transpilation step is required.
    pub async fn eval_typescript(&mut self, code: &str) -> Result<Value> {
        // Check for process.exit() before running
        {
            let exit = self.exit_code.read();
            if let Some(code) = *exit {
                return Err(NodeError::Process(format!(
                    "Process exited with code {}",
                    code
                )));
            }
        }

        // Evaluate the TypeScript code using the engine's native TypeScript mode
        let result = self.engine.eval_typescript(code)?;

        // Run the event loop until no more pending work
        self.run_event_loop().await?;

        Ok(result)
    }

    /// Run a JavaScript file
    pub async fn run_file(&mut self, path: &Path) -> Result<i32> {
        // Resolve the path
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        };

        // Check file exists
        if !abs_path.exists() {
            return Err(NodeError::ModuleNotFound(abs_path.display().to_string()));
        }

        // Determine module type
        let module_type = self.esm_loader.get_module_type(&abs_path)?;

        match module_type {
            ModuleType::ESM => {
                self.run_esm_file(&abs_path).await?;
            }
            ModuleType::CommonJS | ModuleType::Unknown => {
                self.run_cjs_file(&abs_path).await?;
            }
            ModuleType::Json => {
                // JSON files can't be run directly
                return Err(NodeError::TypeError(
                    "Cannot run JSON file directly".to_string(),
                ));
            }
        }

        // Run event loop
        self.run_event_loop().await?;

        // Return exit code
        Ok(self.exit_code.read().unwrap_or(0))
    }

    /// Run a CommonJS file
    async fn run_cjs_file(&mut self, abs_path: &Path) -> Result<()> {
        let dirname = abs_path.parent().unwrap_or(Path::new("."));
        let filename = abs_path;

        // Read and execute the file
        let code = std::fs::read_to_string(abs_path)?;

        // Wrap in module context
        let wrapped = self.wrap_module_code(&code, dirname, filename);

        // Detect TypeScript by extension
        let is_typescript = crate::typescript::is_typescript_file(abs_path);

        // Execute (TypeScript or JavaScript)
        if is_typescript {
            self.engine.eval_typescript(&wrapped)?;
        } else {
            self.engine.eval(&wrapped)?;
        }

        Ok(())
    }

    /// Run an ESM file
    async fn run_esm_file(&mut self, abs_path: &Path) -> Result<()> {
        // Set as main module
        self.esm_loader.set_main_module(abs_path.to_path_buf());

        // Load the module and its dependencies
        let _module = self.esm_loader.load_module(abs_path).await?;

        // Read the source
        let code = std::fs::read_to_string(abs_path)?;

        // Create import.meta for this module
        let import_meta = self.esm_loader.get_import_meta(abs_path);

        // Wrap in ESM context
        let wrapped = self.wrap_esm_code(&code, abs_path, &import_meta);

        // Detect TypeScript by extension
        let is_typescript = crate::typescript::is_typescript_file(abs_path);

        // Execute (TypeScript or JavaScript)
        if is_typescript {
            self.engine.eval_typescript(&wrapped)?;
        } else {
            self.engine.eval(&wrapped)?;
        }

        Ok(())
    }

    /// Wrap code in ESM context with import.meta
    fn wrap_esm_code(
        &self,
        code: &str,
        filename: &Path,
        import_meta: &crate::module_system::ImportMeta,
    ) -> String {
        // For ESM, we need to:
        // 1. Transform imports to require() calls (simplified)
        // 2. Provide import.meta object
        // 3. Handle exports

        // This is a simplified transformation - real implementation would use the AST
        format!(
            r#"(function() {{
    const __importMeta = {{
        url: "{}",
        dirname: "{}",
        filename: "{}",
        main: {},
        resolve: function(specifier) {{
            // TODO: Implement import.meta.resolve
            return specifier;
        }}
    }};
    Object.defineProperty(this, 'import', {{
        value: {{ meta: __importMeta }}
    }});

    // Module code
    {}
}})();"#,
            import_meta.url.replace('\\', "\\\\").replace('"', "\\\""),
            import_meta
                .dirname
                .replace('\\', "\\\\")
                .replace('"', "\\\""),
            import_meta
                .filename
                .replace('\\', "\\\\")
                .replace('"', "\\\""),
            import_meta.main,
            code
        )
    }

    /// Wrap code in a module context with __dirname, __filename, require, etc.
    fn wrap_module_code(&self, code: &str, dirname: &Path, filename: &Path) -> String {
        format!(
            r#"(function(exports, require, module, __filename, __dirname) {{
{}
}})({{}}, require, {{}}, "{}", "{}");"#,
            code,
            filename
                .display()
                .to_string()
                .replace('\\', "\\\\")
                .replace('"', "\\\""),
            dirname
                .display()
                .to_string()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
        )
    }

    /// Run the event loop until no more pending work
    async fn run_event_loop(&mut self) -> Result<()> {
        loop {
            // Check for exit
            {
                let exit = self.exit_code.read();
                if exit.is_some() {
                    break;
                }
            }

            // Process pending callbacks
            let callbacks = self.event_loop.tick();

            if callbacks.is_empty() && !self.event_loop.has_pending_work() {
                break;
            }

            // Execute callbacks
            for callback in callbacks {
                match callback {
                    Callback::Function(func)
                    | Callback::Immediate(func)
                    | Callback::Microtask(func) => {
                        // Execute the callback
                        if let Value::Function(_) = &func {
                            // Call the function through the engine
                            // This is a simplified version - real implementation would
                            // properly invoke the callback
                            tracing::debug!("Executing callback: {:?}", func);
                        }
                    }
                }
            }

            // If we have pending timers, wait for the next one
            if let Some(wait_time) = self.event_loop.time_until_next_timer() {
                if wait_time > Duration::ZERO {
                    tokio::time::sleep(wait_time.min(Duration::from_millis(100))).await;
                }
            } else {
                // Small yield to prevent busy-waiting
                tokio::task::yield_now().await;
            }
        }

        Ok(())
    }

    /// Start the REPL
    pub async fn run_repl(&mut self) -> Result<()> {
        let mut rl = DefaultEditor::new().map_err(|e| NodeError::Generic(e.to_string()))?;

        // Load history
        let history_path = dirs::home_dir()
            .map(|p| p.join(".spacey_node_history"))
            .unwrap_or_else(|| PathBuf::from(".spacey_node_history"));

        let _ = rl.load_history(&history_path);

        loop {
            let prompt = format!("{} ", ">".green().bold());
            match rl.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    // Handle REPL commands
                    if line.starts_with('.') {
                        match line {
                            ".exit" | ".quit" => break,
                            ".help" => {
                                println!("{}", "REPL Commands:".cyan().bold());
                                println!("  {} - Exit the REPL", ".exit".green());
                                println!("  {} - Show this help", ".help".green());
                                println!("  {} - Clear the screen", ".clear".green());
                                println!("  {} - Load and execute a file", ".load <file>".green());
                                continue;
                            }
                            ".clear" => {
                                print!("\x1B[2J\x1B[1;1H");
                                continue;
                            }
                            cmd if cmd.starts_with(".load ") => {
                                let file = cmd.strip_prefix(".load ").unwrap().trim();
                                match self.run_file(Path::new(file)).await {
                                    Ok(_) => println!("{}", "File loaded successfully".green()),
                                    Err(e) => eprintln!("{}: {}", "Error".red(), e),
                                }
                                continue;
                            }
                            _ => {
                                eprintln!("{}: Unknown command '{}'", "Error".red(), line);
                                continue;
                            }
                        }
                    }

                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = rl.add_history_entry(line);

                    // Evaluate
                    match self.eval(line).await {
                        Ok(result) => {
                            if !matches!(result, Value::Undefined) {
                                println!("{}", format_value(&result));
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "Error".red(), e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "(To exit, press Ctrl+D or type .exit)".dimmed());
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(e) => {
                    eprintln!("{}: {:?}", "Readline error".red(), e);
                    break;
                }
            }
        }

        // Save history
        let _ = rl.save_history(&history_path);

        Ok(())
    }

    /// Get a native module by name
    pub fn get_native_module(&self, name: &str) -> Option<&Value> {
        self.native_modules.get(name)
    }

    /// Get the current working directory
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Set the current working directory
    pub fn set_cwd(&mut self, path: PathBuf) {
        self.cwd = path;
    }

    /// Get the event loop
    pub fn event_loop(&self) -> &Arc<EventLoop> {
        &self.event_loop
    }

    /// Get the ESM loader
    pub fn esm_loader(&self) -> &Arc<EsmLoader> {
        &self.esm_loader
    }

    /// Dynamic import (for ESM `import()` expression)
    pub async fn dynamic_import(&self, specifier: &str, parent: &Path) -> Result<Value> {
        crate::module_system::esm::dynamic_import(&self.esm_loader, specifier, parent).await
    }

    /// Check if a file is an ESM module
    pub fn is_esm(&self, path: &Path) -> bool {
        matches!(self.esm_loader.get_module_type(path), Ok(ModuleType::ESM))
    }
}

/// Format a value for REPL output
fn format_value(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".dimmed().to_string(),
        Value::Null => "null".bold().to_string(),
        Value::Boolean(b) => {
            if *b {
                "true".yellow().to_string()
            } else {
                "false".yellow().to_string()
            }
        }
        Value::Number(n) => n.to_string().yellow().to_string(),
        Value::String(s) => format!("'{}'", s).green().to_string(),
        Value::Object(_) => "[Object]".cyan().to_string(),
        Value::Function(_) => "[Function]".magenta().to_string(),
        _ => value.to_string(),
    }
}

impl Default for NodeRuntime {
    fn default() -> Self {
        Self::new(vec![])
    }
}
