// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `dns` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the dns module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Error codes
    exports.insert("NODATA".to_string(), Value::String("ENODATA".to_string()));
    exports.insert("FORMERR".to_string(), Value::String("EFORMERR".to_string()));
    exports.insert(
        "SERVFAIL".to_string(),
        Value::String("ESERVFAIL".to_string()),
    );
    exports.insert(
        "NOTFOUND".to_string(),
        Value::String("ENOTFOUND".to_string()),
    );
    exports.insert("NOTIMP".to_string(), Value::String("ENOTIMP".to_string()));
    exports.insert("REFUSED".to_string(), Value::String("EREFUSED".to_string()));
    exports.insert(
        "BADQUERY".to_string(),
        Value::String("EBADQUERY".to_string()),
    );
    exports.insert("BADNAME".to_string(), Value::String("EBADNAME".to_string()));
    exports.insert(
        "BADFAMILY".to_string(),
        Value::String("EBADFAMILY".to_string()),
    );
    exports.insert("BADRESP".to_string(), Value::String("EBADRESP".to_string()));
    exports.insert(
        "CONNREFUSED".to_string(),
        Value::String("ECONNREFUSED".to_string()),
    );
    exports.insert("TIMEOUT".to_string(), Value::String("ETIMEOUT".to_string()));

    Value::NativeObject(exports)
}
