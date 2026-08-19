// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Built-in Node.js modules
//!
//! Implements core modules like fs, path, http, crypto, etc.

pub mod assert;
pub mod child_process;
pub mod crypto;
pub mod dns;
pub mod events;
pub mod fs;
pub mod http;
pub mod https;
pub mod net;
pub mod os;
pub mod path;
pub mod promises;
pub mod querystring;
pub mod stream;
pub mod string_decoder;
pub mod url;
pub mod util;
pub mod zlib;

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create all native module exports
pub fn create_native_modules() -> HashMap<String, Value> {
    let mut modules = HashMap::new();

    // Core modules
    modules.insert("fs".to_string(), fs::create_module());
    modules.insert("path".to_string(), path::create_module());
    modules.insert("os".to_string(), os::create_module());
    modules.insert("events".to_string(), events::create_module());
    modules.insert("util".to_string(), util::create_module());
    modules.insert("assert".to_string(), assert::create_module());
    modules.insert("querystring".to_string(), querystring::create_module());
    modules.insert(
        "string_decoder".to_string(),
        string_decoder::create_module(),
    );
    modules.insert("url".to_string(), url::create_module());

    // I/O modules
    modules.insert("stream".to_string(), stream::create_module());

    // Network modules
    modules.insert("http".to_string(), http::create_module());
    modules.insert("https".to_string(), https::create_module());
    modules.insert("net".to_string(), net::create_module());
    modules.insert("dns".to_string(), dns::create_module());

    // Security modules
    modules.insert("crypto".to_string(), crypto::create_module());

    // Compression
    modules.insert("zlib".to_string(), zlib::create_module());

    // Process management
    modules.insert("child_process".to_string(), child_process::create_module());

    modules
}

/// Get a built-in module by name
pub fn get_builtin(name: &str) -> Option<Value> {
    let modules = create_native_modules();
    modules.get(name).cloned()
}
