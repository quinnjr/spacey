//! Built-in JavaScript objects and constructors.
//!
//! This module will contain implementations of all built-in objects:
//! - Object, Function, Array, String, Number, Boolean
//! - Math, JSON, Date, RegExp
//! - Map, Set, WeakMap, WeakSet
//! - Promise, Symbol
//! - Error types
//! - TypedArrays, ArrayBuffer, DataView
//! - Etc.

pub mod array;
pub mod collections;
pub mod json;
pub mod native;

pub use json::{json_parse, json_stringify_simple};
pub use native::{BuiltinId, call_builtin};
