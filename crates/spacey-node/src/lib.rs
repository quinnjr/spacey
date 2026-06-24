// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! # spacey-node
//!
//! Node.js bindings for the spacey-spidermonkey JavaScript engine.
//!
//! This crate provides a native Node.js addon that exposes the
//! spacey-spidermonkey engine to Node.js applications.
//!
//! ## Usage (from Node.js)
//!
//! ```javascript
//! const spacey = require('spacey-node');
//!
//! // Create a new engine instance
//! const engine = new spacey.Engine();
//!
//! // Evaluate JavaScript code
//! const result = engine.eval('1 + 2');
//! console.log(result); // 3
//! ```

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use spacey_spidermonkey::{Engine as SpaceyEngine, Value as SpaceyValue};

/// A JavaScript engine instance exposed to Node.js.
#[napi]
pub struct Engine {
    inner: SpaceyEngine,
}

#[napi]
impl Engine {
    /// Creates a new JavaScript engine instance.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SpaceyEngine::new(),
        }
    }

    /// Evaluates JavaScript source code and returns the result.
    #[napi]
    pub fn eval(&mut self, source: String) -> Result<JsValue> {
        match self.inner.eval(&source) {
            Ok(value) => Ok(spacey_value_to_js(value)),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Evaluates JavaScript from a file.
    #[napi]
    pub fn eval_file(&mut self, path: String) -> Result<JsValue> {
        match self.inner.eval_file(std::path::Path::new(&path)) {
            Ok(value) => Ok(spacey_value_to_js(value)),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper for JavaScript values.
#[napi(object)]
pub struct JsValue {
    /// The type of the value
    pub value_type: String,
    /// String representation of the value
    pub value: String,
}

/// Converts a spacey Value to a JsValue for Node.js.
fn spacey_value_to_js(value: SpaceyValue) -> JsValue {
    JsValue {
        value_type: value.type_of().to_string(),
        value: value.to_string(),
    }
}

/// Version information for the spacey-node module.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Gets the name of this engine.
#[napi]
pub fn engine_name() -> String {
    "spacey-spidermonkey".to_string()
}
