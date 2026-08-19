// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `process` global object

use parking_lot::RwLock;
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Create the process object
pub fn create_process_object(
    args: &[String],
    cwd: &Path,
    exit_code: Arc<RwLock<Option<i32>>>,
) -> Value {
    let mut process = HashMap::new();

    // process.argv - Command line arguments (as NativeObject with numeric keys)
    let argv: HashMap<String, Value> = std::iter::once("spacey-node".to_string())
        .chain(args.iter().cloned())
        .enumerate()
        .map(|(i, s)| (i.to_string(), Value::String(s)))
        .collect();
    let mut argv_obj = argv;
    argv_obj.insert("length".to_string(), Value::Number((args.len() + 1) as f64));
    process.insert("argv".to_string(), Value::NativeObject(argv_obj));

    // process.env - Environment variables
    let env: HashMap<String, Value> = std::env::vars()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    process.insert("env".to_string(), Value::NativeObject(env));

    // process.cwd() - Current working directory
    let cwd_str = cwd.display().to_string();
    process.insert("_cwd".to_string(), Value::String(cwd_str));

    // process.pid - Process ID
    process.insert("pid".to_string(), Value::Number(std::process::id() as f64));

    // process.ppid - Parent process ID
    #[cfg(unix)]
    {
        process.insert(
            "ppid".to_string(),
            Value::Number(nix::unistd::getppid().as_raw() as f64),
        );
    }
    #[cfg(not(unix))]
    {
        process.insert("ppid".to_string(), Value::Number(0.0));
    }

    // process.platform
    process.insert(
        "platform".to_string(),
        Value::String(get_platform().to_string()),
    );

    // process.arch
    process.insert("arch".to_string(), Value::String(get_arch().to_string()));

    // process.version
    process.insert(
        "version".to_string(),
        Value::String(format!("v{}", crate::NODE_API_VERSION)),
    );

    // process.versions
    let mut versions = HashMap::new();
    versions.insert(
        "node".to_string(),
        Value::String(crate::NODE_API_VERSION.to_string()),
    );
    versions.insert(
        "spacey".to_string(),
        Value::String(crate::VERSION.to_string()),
    );
    versions.insert(
        "v8".to_string(),
        Value::String("spacey-spidermonkey".to_string()),
    );
    process.insert("versions".to_string(), Value::NativeObject(versions));

    // process.title
    process.insert(
        "title".to_string(),
        Value::String("spacey-node".to_string()),
    );

    // process.execPath
    process.insert(
        "execPath".to_string(),
        Value::String(
            std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "spacey-node".to_string()),
        ),
    );

    // process.execArgv
    let mut exec_argv = HashMap::new();
    exec_argv.insert("length".to_string(), Value::Number(0.0));
    process.insert("execArgv".to_string(), Value::NativeObject(exec_argv));

    // process.hrtime - High-resolution time
    // Will be implemented as a native function

    // process.memoryUsage()
    // Will be implemented as a native function

    // process.uptime()
    // Will be implemented as a native function

    // process.exit()
    // Will be implemented as a native function that sets exit_code

    // process.stdout, process.stderr, process.stdin
    // These would be stream objects - simplified for now
    process.insert("stdout".to_string(), create_stdout_object());
    process.insert("stderr".to_string(), create_stderr_object());
    process.insert("stdin".to_string(), create_stdin_object());

    // process.nextTick()
    // Will be implemented with event loop integration

    Value::NativeObject(process)
}

/// Get the platform string
fn get_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "freebsd") {
        "freebsd"
    } else if cfg!(target_os = "openbsd") {
        "openbsd"
    } else {
        "unknown"
    }
}

/// Get the architecture string
fn get_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "ia32"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        "unknown"
    }
}

/// Create a simplified stdout object
fn create_stdout_object() -> Value {
    let mut stdout = HashMap::new();
    stdout.insert(
        "isTTY".to_string(),
        Value::Boolean(atty::is(atty::Stream::Stdout)),
    );
    stdout.insert("fd".to_string(), Value::Number(1.0));
    // write() method would be a native function
    Value::NativeObject(stdout)
}

/// Create a simplified stderr object
fn create_stderr_object() -> Value {
    let mut stderr = HashMap::new();
    stderr.insert(
        "isTTY".to_string(),
        Value::Boolean(atty::is(atty::Stream::Stderr)),
    );
    stderr.insert("fd".to_string(), Value::Number(2.0));
    Value::NativeObject(stderr)
}

/// Create a simplified stdin object
fn create_stdin_object() -> Value {
    let mut stdin = HashMap::new();
    stdin.insert(
        "isTTY".to_string(),
        Value::Boolean(atty::is(atty::Stream::Stdin)),
    );
    stdin.insert("fd".to_string(), Value::Number(0.0));
    Value::NativeObject(stdin)
}
