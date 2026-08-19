// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Promise implementation for Node.js compatibility
//!
//! Implements ES6+ Promise API with experimental features:
//! - Promise constructor
//! - Promise.resolve, Promise.reject
//! - Promise.all, Promise.race, Promise.allSettled, Promise.any
//! - Promise.withResolvers (ES2024)
//! - then, catch, finally
//! - Async iteration support

use parking_lot::{Mutex, RwLock};
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique ID generator for promises
static PROMISE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Promise state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromiseState {
    /// Initial state, neither fulfilled nor rejected
    Pending,
    /// Operation completed successfully
    Fulfilled,
    /// Operation failed
    Rejected,
}

/// A reaction (then/catch/finally callback)
#[derive(Clone)]
pub struct PromiseReaction {
    /// The promise to resolve with the result
    pub promise: Arc<RwLock<Promise>>,
    /// Fulfillment handler
    pub on_fulfilled: Option<Value>,
    /// Rejection handler
    pub on_rejected: Option<Value>,
    /// Finally handler (called regardless of outcome)
    pub on_finally: Option<Value>,
}

/// Promise implementation
pub struct Promise {
    /// Unique ID
    pub id: u64,
    /// Current state
    pub state: PromiseState,
    /// Resolved value (if fulfilled)
    pub value: Option<Value>,
    /// Rejection reason (if rejected)
    pub reason: Option<Value>,
    /// Pending reactions (callbacks waiting for resolution)
    pub reactions: Vec<PromiseReaction>,
    /// Whether this promise has been handled (for unhandled rejection tracking)
    pub handled: bool,
}

impl Promise {
    /// Create a new pending promise
    pub fn new() -> Self {
        Self {
            id: PROMISE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            state: PromiseState::Pending,
            value: None,
            reason: None,
            reactions: Vec::new(),
            handled: false,
        }
    }

    /// Create a resolved promise
    pub fn resolved(value: Value) -> Self {
        Self {
            id: PROMISE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            state: PromiseState::Fulfilled,
            value: Some(value),
            reason: None,
            reactions: Vec::new(),
            handled: true,
        }
    }

    /// Create a rejected promise
    pub fn rejected(reason: Value) -> Self {
        Self {
            id: PROMISE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            state: PromiseState::Rejected,
            value: None,
            reason: Some(reason),
            reactions: Vec::new(),
            handled: false,
        }
    }

    /// Check if promise is pending
    pub fn is_pending(&self) -> bool {
        self.state == PromiseState::Pending
    }

    /// Check if promise is fulfilled
    pub fn is_fulfilled(&self) -> bool {
        self.state == PromiseState::Fulfilled
    }

    /// Check if promise is rejected
    pub fn is_rejected(&self) -> bool {
        self.state == PromiseState::Rejected
    }

    /// Check if promise is settled (fulfilled or rejected)
    pub fn is_settled(&self) -> bool {
        self.state != PromiseState::Pending
    }

    /// Fulfill the promise with a value
    pub fn fulfill(&mut self, value: Value) {
        if self.state != PromiseState::Pending {
            return; // Already settled
        }
        self.state = PromiseState::Fulfilled;
        self.value = Some(value);

        // Trigger reactions
        let reactions = std::mem::take(&mut self.reactions);
        for reaction in reactions {
            // Queue microtask to handle reaction
            // In real implementation, this would go through the event loop
            tracing::debug!("Promise {} fulfilled, triggering reaction", self.id);
        }
    }

    /// Reject the promise with a reason
    pub fn reject(&mut self, reason: Value) {
        if self.state != PromiseState::Pending {
            return; // Already settled
        }
        self.state = PromiseState::Rejected;
        self.reason = Some(reason);

        // Trigger reactions
        let reactions = std::mem::take(&mut self.reactions);
        if reactions.is_empty() && !self.handled {
            // Unhandled rejection
            tracing::warn!("Unhandled promise rejection: {:?}", self.reason);
        }
        for reaction in reactions {
            tracing::debug!("Promise {} rejected, triggering reaction", self.id);
        }
    }

    /// Add a reaction (then/catch handler)
    pub fn add_reaction(&mut self, reaction: PromiseReaction) {
        if reaction.on_rejected.is_some() {
            self.handled = true;
        }

        match self.state {
            PromiseState::Pending => {
                self.reactions.push(reaction);
            }
            PromiseState::Fulfilled => {
                // Already fulfilled, queue microtask immediately
                tracing::debug!("Promise already fulfilled, queuing reaction");
            }
            PromiseState::Rejected => {
                // Already rejected, queue microtask immediately
                tracing::debug!("Promise already rejected, queuing reaction");
            }
        }
    }
}

impl Default for Promise {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Promise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Promise")
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}

/// Promise capability (for creating promises with external resolve/reject)
#[derive(Clone)]
pub struct PromiseCapability {
    /// The promise
    pub promise: Arc<RwLock<Promise>>,
    /// Resolve function
    pub resolve: Arc<dyn Fn(Value) + Send + Sync>,
    /// Reject function
    pub reject: Arc<dyn Fn(Value) + Send + Sync>,
}

impl PromiseCapability {
    /// Create a new promise capability
    pub fn new() -> Self {
        let promise = Arc::new(RwLock::new(Promise::new()));
        let promise_clone = Arc::clone(&promise);
        let promise_clone2 = Arc::clone(&promise);

        Self {
            promise,
            resolve: Arc::new(move |value| {
                promise_clone.write().fulfill(value);
            }),
            reject: Arc::new(move |reason| {
                promise_clone2.write().reject(reason);
            }),
        }
    }
}

impl Default for PromiseCapability {
    fn default() -> Self {
        Self::new()
    }
}

/// Promise.withResolvers() result (ES2024)
#[derive(Clone)]
pub struct PromiseWithResolvers {
    /// The promise
    pub promise: Arc<RwLock<Promise>>,
    /// Resolve function
    pub resolve: Value,
    /// Reject function
    pub reject: Value,
}

impl PromiseWithResolvers {
    /// Create a new Promise.withResolvers() result
    pub fn new() -> Self {
        let cap = PromiseCapability::new();
        Self {
            promise: cap.promise,
            // In real implementation, these would be callable Value::Function
            resolve: Value::Undefined,
            reject: Value::Undefined,
        }
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert(
            "promise".to_string(),
            Value::String(format!("Promise<{}>", self.promise.read().id)),
        );
        obj.insert("resolve".to_string(), self.resolve.clone());
        obj.insert("reject".to_string(), self.reject.clone());
        Value::NativeObject(obj)
    }
}

impl Default for PromiseWithResolvers {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate error for Promise.any rejections
#[derive(Debug, Clone)]
pub struct AggregateError {
    /// Error message
    pub message: String,
    /// Array of rejection reasons
    pub errors: Vec<Value>,
}

impl AggregateError {
    /// Create a new aggregate error
    pub fn new(message: &str, errors: Vec<Value>) -> Self {
        Self {
            message: message.to_string(),
            errors,
        }
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert(
            "name".to_string(),
            Value::String("AggregateError".to_string()),
        );
        obj.insert("message".to_string(), Value::String(self.message.clone()));

        // errors as array-like object
        let mut errors_obj: HashMap<String, Value> = self
            .errors
            .iter()
            .enumerate()
            .map(|(i, e)| (i.to_string(), e.clone()))
            .collect();
        errors_obj.insert(
            "length".to_string(),
            Value::Number(self.errors.len() as f64),
        );
        obj.insert("errors".to_string(), Value::NativeObject(errors_obj));

        Value::NativeObject(obj)
    }
}

/// Promise.all - wait for all promises to fulfill
pub struct PromiseAll {
    /// Remaining count
    remaining: AtomicU64,
    /// Results
    results: Mutex<Vec<Option<Value>>>,
    /// Result promise
    result: Arc<RwLock<Promise>>,
    /// Whether already rejected
    rejected: std::sync::atomic::AtomicBool,
}

impl PromiseAll {
    /// Create a new Promise.all
    pub fn new(count: usize) -> Self {
        Self {
            remaining: AtomicU64::new(count as u64),
            results: Mutex::new(vec![None; count]),
            result: Arc::new(RwLock::new(Promise::new())),
            rejected: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Called when one promise fulfills
    pub fn on_fulfilled(&self, index: usize, value: Value) {
        if self.rejected.load(Ordering::SeqCst) {
            return;
        }

        {
            let mut results = self.results.lock();
            results[index] = Some(value);
        }

        let remaining = self.remaining.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining == 0 {
            // All fulfilled
            let results = self.results.lock();
            let values: Vec<Value> = results
                .iter()
                .map(|v| v.clone().unwrap_or(Value::Undefined))
                .collect();

            // Convert to array-like object
            let mut arr: HashMap<String, Value> = values
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v))
                .collect();
            arr.insert("length".to_string(), Value::Number(results.len() as f64));

            self.result.write().fulfill(Value::NativeObject(arr));
        }
    }

    /// Called when one promise rejects
    pub fn on_rejected(&self, reason: Value) {
        if self.rejected.swap(true, Ordering::SeqCst) {
            return; // Already rejected
        }
        self.result.write().reject(reason);
    }

    /// Get the result promise
    pub fn promise(&self) -> Arc<RwLock<Promise>> {
        Arc::clone(&self.result)
    }
}

/// Promise.race - first promise to settle wins
pub struct PromiseRace {
    /// Result promise
    result: Arc<RwLock<Promise>>,
    /// Whether already settled
    settled: std::sync::atomic::AtomicBool,
}

impl PromiseRace {
    /// Create a new Promise.race
    pub fn new() -> Self {
        Self {
            result: Arc::new(RwLock::new(Promise::new())),
            settled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Called when one promise fulfills
    pub fn on_fulfilled(&self, value: Value) {
        if self.settled.swap(true, Ordering::SeqCst) {
            return;
        }
        self.result.write().fulfill(value);
    }

    /// Called when one promise rejects
    pub fn on_rejected(&self, reason: Value) {
        if self.settled.swap(true, Ordering::SeqCst) {
            return;
        }
        self.result.write().reject(reason);
    }

    /// Get the result promise
    pub fn promise(&self) -> Arc<RwLock<Promise>> {
        Arc::clone(&self.result)
    }
}

impl Default for PromiseRace {
    fn default() -> Self {
        Self::new()
    }
}

/// Promise.allSettled result
#[derive(Debug, Clone)]
pub struct SettledResult {
    /// "fulfilled" or "rejected"
    pub status: String,
    /// Value if fulfilled
    pub value: Option<Value>,
    /// Reason if rejected
    pub reason: Option<Value>,
}

impl SettledResult {
    /// Create a fulfilled result
    pub fn fulfilled(value: Value) -> Self {
        Self {
            status: "fulfilled".to_string(),
            value: Some(value),
            reason: None,
        }
    }

    /// Create a rejected result
    pub fn rejected(reason: Value) -> Self {
        Self {
            status: "rejected".to_string(),
            value: None,
            reason: Some(reason),
        }
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("status".to_string(), Value::String(self.status.clone()));
        if let Some(v) = &self.value {
            obj.insert("value".to_string(), v.clone());
        }
        if let Some(r) = &self.reason {
            obj.insert("reason".to_string(), r.clone());
        }
        Value::NativeObject(obj)
    }
}

/// Promise.allSettled - wait for all promises to settle
pub struct PromiseAllSettled {
    /// Remaining count
    remaining: AtomicU64,
    /// Results
    results: Mutex<Vec<Option<SettledResult>>>,
    /// Result promise
    result: Arc<RwLock<Promise>>,
}

impl PromiseAllSettled {
    /// Create a new Promise.allSettled
    pub fn new(count: usize) -> Self {
        Self {
            remaining: AtomicU64::new(count as u64),
            results: Mutex::new(vec![None; count]),
            result: Arc::new(RwLock::new(Promise::new())),
        }
    }

    /// Called when one promise settles
    pub fn on_settled(&self, index: usize, result: SettledResult) {
        {
            let mut results = self.results.lock();
            results[index] = Some(result);
        }

        let remaining = self.remaining.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining == 0 {
            // All settled
            let results = self.results.lock();
            let values: Vec<Value> = results
                .iter()
                .map(|r| r.as_ref().map(|s| s.to_value()).unwrap_or(Value::Undefined))
                .collect();

            // Convert to array-like object
            let mut arr: HashMap<String, Value> = values
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v))
                .collect();
            arr.insert("length".to_string(), Value::Number(results.len() as f64));

            self.result.write().fulfill(Value::NativeObject(arr));
        }
    }

    /// Get the result promise
    pub fn promise(&self) -> Arc<RwLock<Promise>> {
        Arc::clone(&self.result)
    }
}

/// Promise.any - first promise to fulfill wins
pub struct PromiseAny {
    /// Remaining count
    remaining: AtomicU64,
    /// Rejection reasons
    errors: Mutex<Vec<Option<Value>>>,
    /// Result promise
    result: Arc<RwLock<Promise>>,
    /// Whether already fulfilled
    fulfilled: std::sync::atomic::AtomicBool,
}

impl PromiseAny {
    /// Create a new Promise.any
    pub fn new(count: usize) -> Self {
        Self {
            remaining: AtomicU64::new(count as u64),
            errors: Mutex::new(vec![None; count]),
            result: Arc::new(RwLock::new(Promise::new())),
            fulfilled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Called when one promise fulfills
    pub fn on_fulfilled(&self, value: Value) {
        if self.fulfilled.swap(true, Ordering::SeqCst) {
            return; // Already fulfilled
        }
        self.result.write().fulfill(value);
    }

    /// Called when one promise rejects
    pub fn on_rejected(&self, index: usize, reason: Value) {
        if self.fulfilled.load(Ordering::SeqCst) {
            return;
        }

        {
            let mut errors = self.errors.lock();
            errors[index] = Some(reason);
        }

        let remaining = self.remaining.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining == 0 {
            // All rejected
            let errors = self.errors.lock();
            let error_values: Vec<Value> = errors
                .iter()
                .map(|e| e.clone().unwrap_or(Value::Undefined))
                .collect();

            let agg_error = AggregateError::new("All promises were rejected", error_values);
            self.result.write().reject(agg_error.to_value());
        }
    }

    /// Get the result promise
    pub fn promise(&self) -> Arc<RwLock<Promise>> {
        Arc::clone(&self.result)
    }
}

/// Create the Promise module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Promise constructor would be a native function
    // Static methods markers
    exports.insert(
        "resolve".to_string(),
        Value::String("__native__Promise.resolve".to_string()),
    );
    exports.insert(
        "reject".to_string(),
        Value::String("__native__Promise.reject".to_string()),
    );
    exports.insert(
        "all".to_string(),
        Value::String("__native__Promise.all".to_string()),
    );
    exports.insert(
        "race".to_string(),
        Value::String("__native__Promise.race".to_string()),
    );
    exports.insert(
        "allSettled".to_string(),
        Value::String("__native__Promise.allSettled".to_string()),
    );
    exports.insert(
        "any".to_string(),
        Value::String("__native__Promise.any".to_string()),
    );
    exports.insert(
        "withResolvers".to_string(),
        Value::String("__native__Promise.withResolvers".to_string()),
    );

    Value::NativeObject(exports)
}

/// Async iterator protocol support
pub mod async_iter {
    use super::*;

    /// Async iterator result
    #[derive(Debug, Clone)]
    pub struct AsyncIteratorResult {
        /// The yielded value
        pub value: Value,
        /// Whether iteration is complete
        pub done: bool,
    }

    impl AsyncIteratorResult {
        /// Create a new result
        pub fn new(value: Value, done: bool) -> Self {
            Self { value, done }
        }

        /// Create a "done" result
        pub fn done() -> Self {
            Self {
                value: Value::Undefined,
                done: true,
            }
        }

        /// Convert to JavaScript Value
        pub fn to_value(&self) -> Value {
            let mut obj = HashMap::new();
            obj.insert("value".to_string(), self.value.clone());
            obj.insert("done".to_string(), Value::Boolean(self.done));
            Value::NativeObject(obj)
        }
    }

    /// Async iterable protocol
    pub trait AsyncIterable {
        /// Get the async iterator
        fn async_iterator(&self) -> Box<dyn AsyncIterator>;
    }

    /// Async iterator protocol
    pub trait AsyncIterator: Send + Sync {
        /// Get the next value (returns a promise)
        fn next(&mut self) -> Arc<RwLock<Promise>>;

        /// Return early (optional)
        fn return_value(&mut self, _value: Value) -> Arc<RwLock<Promise>> {
            let promise = Arc::new(RwLock::new(Promise::new()));
            promise
                .write()
                .fulfill(AsyncIteratorResult::done().to_value());
            promise
        }

        /// Throw an error (optional)
        fn throw(&mut self, error: Value) -> Arc<RwLock<Promise>> {
            let promise = Arc::new(RwLock::new(Promise::new()));
            promise.write().reject(error);
            promise
        }
    }

    /// Async generator state
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AsyncGeneratorState {
        SuspendedStart,
        SuspendedYield,
        Executing,
        AwaitingReturn,
        Completed,
    }

    /// Simple async from sync iterator wrapper
    pub struct AsyncFromSyncIterator {
        /// Values to iterate
        values: Vec<Value>,
        /// Current index
        index: usize,
    }

    impl AsyncFromSyncIterator {
        /// Create from an array of values
        pub fn new(values: Vec<Value>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl AsyncIterator for AsyncFromSyncIterator {
        fn next(&mut self) -> Arc<RwLock<Promise>> {
            let promise = Arc::new(RwLock::new(Promise::new()));

            if self.index < self.values.len() {
                let value = self.values[self.index].clone();
                self.index += 1;
                let result = AsyncIteratorResult::new(value, false);
                promise.write().fulfill(result.to_value());
            } else {
                promise
                    .write()
                    .fulfill(AsyncIteratorResult::done().to_value());
            }

            promise
        }
    }
}

/// for-await-of support
pub mod for_await {
    use super::async_iter::*;
    use super::*;

    /// Execute a for-await-of loop
    pub async fn for_await_of<F>(
        iterator: &mut dyn AsyncIterator,
        mut callback: F,
    ) -> Result<(), Value>
    where
        F: FnMut(Value) -> Result<(), Value>,
    {
        loop {
            let next_promise = iterator.next();

            // Wait for the promise to settle
            // In real implementation, this would yield to the event loop
            let next_result = {
                let promise = next_promise.read();
                if promise.is_fulfilled() {
                    promise.value.clone()
                } else if promise.is_rejected() {
                    return Err(promise.reason.clone().unwrap_or(Value::Undefined));
                } else {
                    // Still pending - would need to await
                    continue;
                }
            };

            if let Some(result) = next_result {
                // Parse the iterator result
                if let Value::NativeObject(obj) = &result {
                    let done = obj
                        .get("done")
                        .map(|v| matches!(v, Value::Boolean(true)))
                        .unwrap_or(false);

                    if done {
                        break;
                    }

                    let value = obj.get("value").cloned().unwrap_or(Value::Undefined);
                    callback(value)?;
                }
            }
        }

        Ok(())
    }
}

/// Top-level await support marker
pub struct TopLevelAwait {
    /// Whether TLA is enabled
    pub enabled: bool,
    /// Pending TLA promises
    pub pending: Vec<Arc<RwLock<Promise>>>,
}

impl TopLevelAwait {
    /// Create new TLA context
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            pending: Vec::new(),
        }
    }

    /// Add a pending TLA promise
    pub fn add_pending(&mut self, promise: Arc<RwLock<Promise>>) {
        self.pending.push(promise);
    }

    /// Check if all TLA promises are settled
    pub fn all_settled(&self) -> bool {
        self.pending.iter().all(|p| p.read().is_settled())
    }

    /// Wait for all TLA promises
    pub async fn wait_all(&self) -> Result<(), Value> {
        for promise in &self.pending {
            // Wait for each promise
            loop {
                let state = {
                    let p = promise.read();
                    if p.is_fulfilled() {
                        break;
                    } else if p.is_rejected() {
                        return Err(p.reason.clone().unwrap_or(Value::Undefined));
                    }
                    p.state.clone()
                };

                if state == PromiseState::Pending {
                    // Yield to event loop
                    tokio::task::yield_now().await;
                }
            }
        }
        Ok(())
    }
}

impl Default for TopLevelAwait {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_creation() {
        let promise = Promise::new();
        assert!(promise.is_pending());
        assert!(!promise.is_settled());
    }

    #[test]
    fn test_promise_fulfill() {
        let mut promise = Promise::new();
        promise.fulfill(Value::Number(42.0));

        assert!(promise.is_fulfilled());
        assert!(promise.is_settled());
        assert_eq!(promise.value, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_promise_reject() {
        let mut promise = Promise::new();
        promise.reject(Value::String("error".to_string()));

        assert!(promise.is_rejected());
        assert!(promise.is_settled());
        assert_eq!(promise.reason, Some(Value::String("error".to_string())));
    }

    #[test]
    fn test_promise_resolved() {
        let promise = Promise::resolved(Value::Number(42.0));
        assert!(promise.is_fulfilled());
    }

    #[test]
    fn test_promise_rejected() {
        let promise = Promise::rejected(Value::String("error".to_string()));
        assert!(promise.is_rejected());
    }

    #[test]
    fn test_promise_capability() {
        let cap = PromiseCapability::new();
        assert!(cap.promise.read().is_pending());

        (cap.resolve)(Value::Number(42.0));
        assert!(cap.promise.read().is_fulfilled());
    }

    #[test]
    fn test_settled_result() {
        let fulfilled = SettledResult::fulfilled(Value::Number(42.0));
        assert_eq!(fulfilled.status, "fulfilled");

        let rejected = SettledResult::rejected(Value::String("error".to_string()));
        assert_eq!(rejected.status, "rejected");
    }

    #[test]
    fn test_aggregate_error() {
        let errors = vec![
            Value::String("error1".to_string()),
            Value::String("error2".to_string()),
        ];
        let agg = AggregateError::new("All failed", errors);
        assert_eq!(agg.message, "All failed");
        assert_eq!(agg.errors.len(), 2);
    }

    #[test]
    fn test_promise_with_resolvers() {
        let pwr = PromiseWithResolvers::new();
        assert!(pwr.promise.read().is_pending());
    }
}
