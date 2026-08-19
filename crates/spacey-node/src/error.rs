// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Error types for the Node.js runtime

use std::path::PathBuf;
use thiserror::Error;

/// Result type for Node.js runtime operations
pub type Result<T> = std::result::Result<T, NodeError>;

/// Errors that can occur in the Node.js runtime
#[derive(Debug, Error)]
pub enum NodeError {
    /// JavaScript engine error
    #[error("{0}")]
    Engine(#[from] spacey_spidermonkey::Error),

    /// Module not found
    #[error("Cannot find module '{0}'")]
    ModuleNotFound(String),

    /// Module resolution error
    #[error("Error resolving module '{module}': {reason}")]
    ModuleResolution {
        /// Module specifier
        module: String,
        /// Reason for failure
        reason: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// File system error
    #[error("File system error: {0}")]
    Fs(#[from] std::io::Error),

    /// Path error
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Crypto error
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Process error
    #[error("Process error: {0}")]
    Process(String),

    /// Type error (wrong argument type)
    #[error("TypeError: {0}")]
    TypeError(String),

    /// Range error (value out of range)
    #[error("RangeError: {0}")]
    RangeError(String),

    /// Reference error (undefined variable)
    #[error("ReferenceError: {0}")]
    ReferenceError(String),

    /// Syntax error
    #[error("SyntaxError: {0}")]
    SyntaxError(String),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Assertion error
    #[error("AssertionError: {0}")]
    Assertion(String),

    /// Event error
    #[error("EventError: {0}")]
    Event(String),
}

impl NodeError {
    /// Create a new TypeError
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::TypeError(msg.into())
    }

    /// Create a new RangeError
    pub fn range_error(msg: impl Into<String>) -> Self {
        Self::RangeError(msg.into())
    }

    /// Create a new ReferenceError
    pub fn reference_error(msg: impl Into<String>) -> Self {
        Self::ReferenceError(msg.into())
    }

    /// Create a module not found error
    pub fn module_not_found(module: impl Into<String>) -> Self {
        Self::ModuleNotFound(module.into())
    }
}
