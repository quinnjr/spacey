// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `child_process` module implementation

use crate::error::{NodeError, Result};
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::process::{Command, Output, Stdio};

/// Create the child_process module exports
pub fn create_module() -> Value {
    let exports = HashMap::new();
    Value::NativeObject(exports)
}

/// Execute a command synchronously
pub fn exec_sync(command: &str, options: Option<ExecOptions>) -> Result<Vec<u8>> {
    let options = options.unwrap_or_default();

    let shell = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut cmd = Command::new(shell.0);
    cmd.arg(shell.1).arg(command);

    if let Some(cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    if let Some(env) = options.env {
        cmd.envs(env);
    }

    if options.stdio == "pipe" {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    }

    let output = cmd
        .output()
        .map_err(|e| NodeError::Process(format!("Failed to execute command: {}", e)))?;

    if !output.status.success() && !options.ignore_errors {
        return Err(NodeError::Process(format!(
            "Command failed with exit code: {:?}",
            output.status.code()
        )));
    }

    Ok(output.stdout)
}

/// Execute a file synchronously
pub fn exec_file_sync(file: &str, args: &[&str], options: Option<ExecOptions>) -> Result<Vec<u8>> {
    let options = options.unwrap_or_default();

    let mut cmd = Command::new(file);
    cmd.args(args);

    if let Some(cwd) = options.cwd {
        cmd.current_dir(cwd);
    }

    if let Some(env) = options.env {
        cmd.envs(env);
    }

    if options.stdio == "pipe" {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    }

    let output = cmd
        .output()
        .map_err(|e| NodeError::Process(format!("Failed to execute file: {}", e)))?;

    if !output.status.success() && !options.ignore_errors {
        return Err(NodeError::Process(format!(
            "Command failed with exit code: {:?}",
            output.status.code()
        )));
    }

    Ok(output.stdout)
}

/// Spawn a child process synchronously
pub fn spawn_sync(
    command: &str,
    args: &[&str],
    options: Option<SpawnOptions>,
) -> Result<SpawnResult> {
    let options = options.unwrap_or_default();

    let mut cmd = Command::new(command);
    cmd.args(args);

    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    if let Some(env) = &options.env {
        cmd.envs(env.clone());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| NodeError::Process(format!("Failed to spawn process: {}", e)))?;

    Ok(SpawnResult {
        stdout: output.stdout,
        stderr: output.stderr,
        status: output.status.code(),
        signal: None,
        error: None,
    })
}

/// Execute options
#[derive(Debug, Default)]
pub struct ExecOptions {
    /// Current working directory
    pub cwd: Option<String>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Encoding
    pub encoding: Option<String>,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
    /// Max buffer size
    pub max_buffer: Option<usize>,
    /// Kill signal
    pub kill_signal: Option<String>,
    /// Shell to use
    pub shell: Option<String>,
    /// stdio configuration
    pub stdio: String,
    /// Ignore errors
    pub ignore_errors: bool,
}

/// Spawn options
#[derive(Debug, Default)]
pub struct SpawnOptions {
    /// Current working directory
    pub cwd: Option<String>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Arguments
    pub argv0: Option<String>,
    /// stdio configuration
    pub stdio: Option<String>,
    /// Detached process
    pub detached: bool,
    /// User ID
    pub uid: Option<u32>,
    /// Group ID
    pub gid: Option<u32>,
    /// Shell
    pub shell: Option<String>,
    /// Windows verbatim arguments
    pub windows_verbatim_arguments: bool,
    /// Windows hide
    pub windows_hide: bool,
}

/// Result of spawnSync
#[derive(Debug)]
pub struct SpawnResult {
    /// stdout data
    pub stdout: Vec<u8>,
    /// stderr data
    pub stderr: Vec<u8>,
    /// Exit status
    pub status: Option<i32>,
    /// Signal that killed the process
    pub signal: Option<String>,
    /// Error if spawn failed
    pub error: Option<String>,
}

impl SpawnResult {
    /// Convert to JavaScript value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        // stdout as array-like object
        let mut stdout_obj: HashMap<String, Value> = self
            .stdout
            .iter()
            .enumerate()
            .map(|(i, &b)| (i.to_string(), Value::Number(b as f64)))
            .collect();
        stdout_obj.insert(
            "length".to_string(),
            Value::Number(self.stdout.len() as f64),
        );
        obj.insert("stdout".to_string(), Value::NativeObject(stdout_obj));

        // stderr as array-like object
        let mut stderr_obj: HashMap<String, Value> = self
            .stderr
            .iter()
            .enumerate()
            .map(|(i, &b)| (i.to_string(), Value::Number(b as f64)))
            .collect();
        stderr_obj.insert(
            "length".to_string(),
            Value::Number(self.stderr.len() as f64),
        );
        obj.insert("stderr".to_string(), Value::NativeObject(stderr_obj));
        obj.insert(
            "status".to_string(),
            self.status
                .map(|s| Value::Number(s as f64))
                .unwrap_or(Value::Null),
        );
        obj.insert(
            "signal".to_string(),
            self.signal
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        if let Some(err) = &self.error {
            obj.insert("error".to_string(), Value::String(err.clone()));
        }
        Value::NativeObject(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_sync() {
        let result = exec_sync("echo hello", None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(String::from_utf8_lossy(&output).contains("hello"));
    }

    #[test]
    fn test_spawn_sync() {
        let result = spawn_sync("echo", &["hello"], None);
        assert!(result.is_ok());
        let spawn_result = result.unwrap();
        assert!(String::from_utf8_lossy(&spawn_result.stdout).contains("hello"));
    }
}
