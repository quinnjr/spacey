// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js global objects and functions
//!
//! Implements:
//! - `process` - Process information and control
//! - `Buffer` - Binary data handling
//! - `console` - Enhanced console output
//! - `setTimeout`, `setInterval`, `setImmediate` - Timer functions
//! - `clearTimeout`, `clearInterval`, `clearImmediate` - Timer cancellation
//! - `queueMicrotask` - Microtask scheduling
//! - `global` - The global object
//! - `__dirname`, `__filename` - Module path information

pub mod buffer;
pub mod console;
pub mod process;
pub mod timers;

use crate::runtime::EventLoop;
use parking_lot::RwLock;
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Create all Node.js globals
pub fn create_globals(
    args: &[String],
    cwd: &Path,
    event_loop: Arc<EventLoop>,
    exit_code: Arc<RwLock<Option<i32>>>,
) -> Value {
    let mut globals = HashMap::new();

    // process object
    globals.insert(
        "process".to_string(),
        process::create_process_object(args, cwd, Arc::clone(&exit_code)),
    );

    // Buffer class
    globals.insert("Buffer".to_string(), buffer::create_buffer_class());

    // Console (enhanced)
    globals.insert("console".to_string(), console::create_console_object());

    // Timer functions
    let timer_funcs = timers::create_timer_functions(event_loop);
    for (name, func) in timer_funcs {
        globals.insert(name, func);
    }

    // global object (reference to itself)
    globals.insert("global".to_string(), Value::Undefined); // Will be set to the global object

    // globalThis (ES2020)
    globals.insert("globalThis".to_string(), Value::Undefined);

    Value::NativeObject(globals)
}

/// Create the `global` object reference
pub fn create_global_object() -> Value {
    // This returns a reference to the global scope
    // In practice, this would be handled by the engine
    Value::Undefined
}
