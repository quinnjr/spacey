// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `https` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the https module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Re-use most of http module functionality with TLS
    exports.insert("globalAgent".to_string(), Value::Undefined);

    Value::NativeObject(exports)
}
