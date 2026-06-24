//! Proxy and Reflect implementations.
//!
//! Note: This is a simplified implementation. A full implementation would
//! integrate proxy traps with property access in the interpreter.

/// The set of trap names a `Proxy` handler may define.
pub const TRAP_NAMES: &[&str] = &[
    "get",
    "set",
    "has",
    "deleteProperty",
    "ownKeys",
    "getOwnPropertyDescriptor",
    "defineProperty",
    "getPrototypeOf",
    "setPrototypeOf",
    "isExtensible",
    "preventExtensions",
    "apply",
    "construct",
];

/// A `Proxy` wrapping a target object with a handler object.
#[derive(Debug, Clone)]
pub struct Proxy {
    /// The wrapped target object reference.
    target: usize,
    /// The handler object reference holding the traps.
    handler: usize,
}

impl Proxy {
    /// Creates a new `Proxy` for the given target and handler object references.
    pub fn new(target: usize, handler: usize) -> Self {
        Self { target, handler }
    }

    /// Returns the wrapped target object reference.
    pub fn target(&self) -> usize {
        self.target
    }

    /// Returns the handler object reference.
    pub fn handler(&self) -> usize {
        self.handler
    }
}
