//! Symbol primitive implementation.
//!
//! Note: This is a simplified implementation. A full implementation would
//! integrate symbols with property keys throughout the interpreter.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing source of unique symbol identifiers.
static NEXT_SYMBOL_ID: AtomicU64 = AtomicU64::new(1);

/// A unique `Symbol` primitive value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// The globally unique identifier for this symbol.
    id: u64,
    /// The optional description provided at creation time.
    description: Option<String>,
}

impl Symbol {
    /// Creates a new unique `Symbol` with an optional description.
    pub fn new(description: Option<String>) -> Self {
        Self {
            id: NEXT_SYMBOL_ID.fetch_add(1, Ordering::Relaxed),
            description,
        }
    }

    /// Returns the unique identifier of this symbol.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the symbol's description, if any.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// The global symbol registry backing `Symbol.for` / `Symbol.keyFor`.
#[derive(Debug, Default)]
pub struct SymbolRegistry {
    by_key: HashMap<String, Symbol>,
}

impl SymbolRegistry {
    /// Creates an empty symbol registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the symbol registered for `key`, creating it if necessary.
    pub fn for_key(&mut self, key: &str) -> Symbol {
        self.by_key
            .entry(key.to_string())
            .or_insert_with(|| Symbol::new(Some(key.to_string())))
            .clone()
    }

    /// Returns the registry key for `symbol`, if it was registered via [`for_key`].
    ///
    /// [`for_key`]: SymbolRegistry::for_key
    pub fn key_for(&self, symbol: &Symbol) -> Option<&str> {
        self.by_key
            .iter()
            .find(|(_, s)| *s == symbol)
            .map(|(k, _)| k.as_str())
    }
}
