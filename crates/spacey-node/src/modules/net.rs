// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `net` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the net module exports
pub fn create_module() -> Value {
    let exports = HashMap::new();
    Value::NativeObject(exports)
}

/// Check if input is an IP address
pub fn is_ip(input: &str) -> i32 {
    if is_ipv4(input) {
        4
    } else if is_ipv6(input) {
        6
    } else {
        0
    }
}

/// Check if input is an IPv4 address
pub fn is_ipv4(input: &str) -> bool {
    input.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Check if input is an IPv6 address
pub fn is_ipv6(input: &str) -> bool {
    input.parse::<std::net::Ipv6Addr>().is_ok()
}
