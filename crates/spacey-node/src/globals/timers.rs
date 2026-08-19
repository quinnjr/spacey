// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Timer functions: setTimeout, setInterval, setImmediate, etc.

use crate::runtime::EventLoop;
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Create all timer-related global functions
pub fn create_timer_functions(event_loop: Arc<EventLoop>) -> Vec<(String, Value)> {
    let mut functions = Vec::new();

    // These would be native functions that interact with the event loop
    // For now, we return placeholder values

    // setTimeout(callback, delay, ...args)
    functions.push(("setTimeout".to_string(), Value::Undefined));

    // clearTimeout(timeoutId)
    functions.push(("clearTimeout".to_string(), Value::Undefined));

    // setInterval(callback, delay, ...args)
    functions.push(("setInterval".to_string(), Value::Undefined));

    // clearInterval(intervalId)
    functions.push(("clearInterval".to_string(), Value::Undefined));

    // setImmediate(callback, ...args)
    functions.push(("setImmediate".to_string(), Value::Undefined));

    // clearImmediate(immediateId)
    functions.push(("clearImmediate".to_string(), Value::Undefined));

    // queueMicrotask(callback)
    functions.push(("queueMicrotask".to_string(), Value::Undefined));

    functions
}

/// Timeout handle returned by setTimeout
#[derive(Clone)]
pub struct Timeout {
    /// The timer ID
    pub id: u64,
    /// Reference to refresh the timer
    refresh_ref: Arc<EventLoop>,
}

impl std::fmt::Debug for Timeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timeout").field("id", &self.id).finish()
    }
}

impl Timeout {
    /// Create a new timeout handle
    pub fn new(id: u64, event_loop: Arc<EventLoop>) -> Self {
        Self {
            id,
            refresh_ref: event_loop,
        }
    }

    /// Refresh the timer (reset its deadline)
    pub fn refresh(&self) {
        // Re-schedule the timer with the same ID
        // This would need to be implemented in the event loop
    }

    /// Check if the timer has been executed
    pub fn has_ref(&self) -> bool {
        // Check if the timer is still active
        true
    }

    /// Prevent the timer from keeping the process alive
    pub fn unref(&self) {
        // Mark the timer as not keeping the process alive
    }

    /// Allow the timer to keep the process alive (default)
    pub fn ref_(&self) {
        // Mark the timer as keeping the process alive
    }
}

/// Interval handle returned by setInterval
#[derive(Clone)]
pub struct Interval {
    /// The timer ID
    pub id: u64,
    /// Reference to the event loop
    event_loop: Arc<EventLoop>,
}

impl std::fmt::Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interval").field("id", &self.id).finish()
    }
}

impl Interval {
    /// Create a new interval handle
    pub fn new(id: u64, event_loop: Arc<EventLoop>) -> Self {
        Self { id, event_loop }
    }

    /// Check if the interval has a reference
    pub fn has_ref(&self) -> bool {
        true
    }

    /// Prevent the interval from keeping the process alive
    pub fn unref(&self) {
        // Mark as not keeping process alive
    }

    /// Allow the interval to keep the process alive
    pub fn ref_(&self) {
        // Mark as keeping process alive
    }
}

/// Immediate handle returned by setImmediate
#[derive(Debug, Clone)]
pub struct Immediate {
    /// The immediate ID
    pub id: u64,
}

impl Immediate {
    /// Create a new immediate handle
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Check if the immediate has a reference
    pub fn has_ref(&self) -> bool {
        true
    }

    /// Prevent the immediate from keeping the process alive
    pub fn unref(&self) {
        // Mark as not keeping process alive
    }

    /// Allow the immediate to keep the process alive
    pub fn ref_(&self) {
        // Mark as keeping process alive
    }
}
