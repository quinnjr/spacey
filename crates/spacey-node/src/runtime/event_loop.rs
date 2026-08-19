// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Event loop implementation (libuv equivalent)
//!
//! Provides async I/O scheduling, timers, and callback management.

use crate::modules::promises::{Promise, PromiseCapability};
use parking_lot::{Mutex, RwLock};
use spacey_spidermonkey::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Unique identifier for a timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerId(pub u64);

/// A scheduled timer
#[derive(Debug)]
pub struct Timer {
    /// Unique timer ID
    pub id: TimerId,
    /// When the timer should fire
    pub deadline: Instant,
    /// Callback to execute
    pub callback: Value,
    /// Whether this is a repeating interval
    pub repeat: Option<Duration>,
    /// Whether the timer has been cancelled
    pub cancelled: bool,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (earliest deadline first)
        other.deadline.cmp(&self.deadline)
    }
}

/// Callback types that can be scheduled
#[derive(Debug)]
pub enum Callback {
    /// JavaScript function callback
    Function(Value),
    /// Immediate callback (setImmediate)
    Immediate(Value),
    /// Microtask (Promise resolution, queueMicrotask)
    Microtask(Value),
}

/// Message types for the event loop
#[derive(Debug)]
pub enum EventMessage {
    /// Schedule a new timer
    ScheduleTimer(Timer),
    /// Cancel a timer
    CancelTimer(TimerId),
    /// Queue an immediate callback
    QueueImmediate(Value),
    /// Queue a microtask
    QueueMicrotask(Value),
    /// Queue a promise microtask (higher priority)
    QueuePromiseMicrotask(Value),
    /// Signal to stop the event loop
    Stop,
}

/// Promise job for the microtask queue
#[derive(Debug)]
pub struct PromiseJob {
    /// The callback to execute
    pub callback: Value,
    /// Arguments to pass to the callback
    pub arguments: Vec<Value>,
}

/// The main event loop
pub struct EventLoop {
    /// Timer counter for generating unique IDs
    next_timer_id: AtomicU64,
    /// Pending timers (min-heap by deadline)
    timers: Arc<Mutex<BinaryHeap<Timer>>>,
    /// Pending immediate callbacks
    immediates: Arc<Mutex<VecDeque<Value>>>,
    /// Microtask queue (queueMicrotask)
    microtasks: Arc<Mutex<VecDeque<Value>>>,
    /// Promise microtask queue (Promise reactions - higher priority)
    promise_microtasks: Arc<Mutex<VecDeque<Value>>>,
    /// Channel for sending events to the loop
    event_tx: mpsc::UnboundedSender<EventMessage>,
    /// Channel for receiving events
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<EventMessage>>>,
    /// Whether the loop is running
    running: AtomicBool,
    /// Whether we have any pending work
    has_pending_work: AtomicBool,
    /// Pending promises (for tracking unhandled rejections)
    pending_promises: Arc<Mutex<Vec<Arc<RwLock<Promise>>>>>,
    /// Unhandled rejection callback
    unhandled_rejection_handler: Arc<Mutex<Option<Value>>>,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            next_timer_id: AtomicU64::new(1),
            timers: Arc::new(Mutex::new(BinaryHeap::new())),
            immediates: Arc::new(Mutex::new(VecDeque::new())),
            microtasks: Arc::new(Mutex::new(VecDeque::new())),
            promise_microtasks: Arc::new(Mutex::new(VecDeque::new())),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            running: AtomicBool::new(false),
            has_pending_work: AtomicBool::new(false),
            pending_promises: Arc::new(Mutex::new(Vec::new())),
            unhandled_rejection_handler: Arc::new(Mutex::new(None)),
        }
    }

    /// Generate a new unique timer ID
    fn next_id(&self) -> TimerId {
        TimerId(self.next_timer_id.fetch_add(1, AtomicOrdering::SeqCst))
    }

    /// Schedule a timeout (setTimeout)
    pub fn set_timeout(&self, callback: Value, delay_ms: u64) -> TimerId {
        let id = self.next_id();
        let timer = Timer {
            id,
            deadline: Instant::now() + Duration::from_millis(delay_ms),
            callback,
            repeat: None,
            cancelled: false,
        };

        let _ = self.event_tx.send(EventMessage::ScheduleTimer(timer));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);
        id
    }

    /// Schedule an interval (setInterval)
    pub fn set_interval(&self, callback: Value, interval_ms: u64) -> TimerId {
        let id = self.next_id();
        let interval = Duration::from_millis(interval_ms);
        let timer = Timer {
            id,
            deadline: Instant::now() + interval,
            callback,
            repeat: Some(interval),
            cancelled: false,
        };

        let _ = self.event_tx.send(EventMessage::ScheduleTimer(timer));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);
        id
    }

    /// Cancel a timer (clearTimeout/clearInterval)
    pub fn clear_timer(&self, id: TimerId) {
        let _ = self.event_tx.send(EventMessage::CancelTimer(id));
    }

    /// Queue an immediate callback (setImmediate)
    pub fn set_immediate(&self, callback: Value) -> TimerId {
        let id = self.next_id();
        let _ = self.event_tx.send(EventMessage::QueueImmediate(callback));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);
        id
    }

    /// Queue a microtask (queueMicrotask)
    pub fn queue_microtask(&self, callback: Value) {
        let _ = self.event_tx.send(EventMessage::QueueMicrotask(callback));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);
    }

    /// Queue a promise microtask (Promise resolution - higher priority)
    pub fn queue_promise_microtask(&self, callback: Value) {
        let _ = self
            .event_tx
            .send(EventMessage::QueuePromiseMicrotask(callback));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);
    }

    /// Create a promise that resolves after a delay (like util.promisify(setTimeout))
    pub fn delay(&self, ms: u64) -> Arc<RwLock<Promise>> {
        let promise = Arc::new(RwLock::new(Promise::new()));
        let promise_clone = Arc::clone(&promise);

        // Schedule a timer that fulfills the promise
        let id = self.next_id();
        let timer = Timer {
            id,
            deadline: Instant::now() + Duration::from_millis(ms),
            callback: Value::Undefined, // Would be replaced with actual callback
            repeat: None,
            cancelled: false,
        };
        let _ = self.event_tx.send(EventMessage::ScheduleTimer(timer));
        self.has_pending_work.store(true, AtomicOrdering::SeqCst);

        // Track for fulfillment
        self.pending_promises.lock().push(promise_clone);

        promise
    }

    /// Resolve a value (Promise.resolve)
    pub fn resolve(&self, value: Value) -> Arc<RwLock<Promise>> {
        Arc::new(RwLock::new(Promise::resolved(value)))
    }

    /// Reject with a reason (Promise.reject)
    pub fn reject(&self, reason: Value) -> Arc<RwLock<Promise>> {
        Arc::new(RwLock::new(Promise::rejected(reason)))
    }

    /// Set the unhandled rejection handler
    pub fn set_unhandled_rejection_handler(&self, handler: Value) {
        *self.unhandled_rejection_handler.lock() = Some(handler);
    }

    /// Handle an unhandled promise rejection
    pub fn emit_unhandled_rejection(&self, promise: &Promise, reason: &Value) {
        if let Some(handler) = self.unhandled_rejection_handler.lock().as_ref() {
            // Queue a microtask to call the handler
            tracing::warn!(
                "Unhandled promise rejection (promise id: {}): {:?}",
                promise.id,
                reason
            );
        }
    }

    /// Track a promise for unhandled rejection detection
    pub fn track_promise(&self, promise: Arc<RwLock<Promise>>) {
        self.pending_promises.lock().push(promise);
    }

    /// Check for unhandled rejections (called at end of tick)
    pub fn check_unhandled_rejections(&self) {
        let promises = self.pending_promises.lock();
        for promise in promises.iter() {
            let p = promise.read();
            if p.is_rejected() && !p.handled {
                if let Some(reason) = &p.reason {
                    self.emit_unhandled_rejection(&p, reason);
                }
            }
        }
    }

    /// Process pending events and return callbacks ready to execute
    pub fn tick(&self) -> Vec<Callback> {
        let mut ready_callbacks = Vec::new();

        // Process incoming events
        {
            let mut rx = self.event_rx.lock();
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    EventMessage::ScheduleTimer(timer) => {
                        self.timers.lock().push(timer);
                    }
                    EventMessage::CancelTimer(id) => {
                        let mut timers = self.timers.lock();
                        // Mark timer as cancelled (will be skipped when popped)
                        let heap: Vec<_> = std::mem::take(&mut *timers).into_vec();
                        for mut timer in heap {
                            if timer.id == id {
                                timer.cancelled = true;
                            }
                            timers.push(timer);
                        }
                    }
                    EventMessage::QueueImmediate(callback) => {
                        self.immediates.lock().push_back(callback);
                    }
                    EventMessage::QueueMicrotask(callback) => {
                        self.microtasks.lock().push_back(callback);
                    }
                    EventMessage::QueuePromiseMicrotask(callback) => {
                        self.promise_microtasks.lock().push_back(callback);
                    }
                    EventMessage::Stop => {
                        self.running.store(false, AtomicOrdering::SeqCst);
                    }
                }
            }
        }

        // Process promise microtasks first (highest priority - spec compliance)
        {
            let mut promise_microtasks = self.promise_microtasks.lock();
            while let Some(callback) = promise_microtasks.pop_front() {
                ready_callbacks.push(Callback::Microtask(callback));
            }
        }

        // Process regular microtasks second
        {
            let mut microtasks = self.microtasks.lock();
            while let Some(callback) = microtasks.pop_front() {
                ready_callbacks.push(Callback::Microtask(callback));
            }
        }

        // Process immediates
        {
            let mut immediates = self.immediates.lock();
            while let Some(callback) = immediates.pop_front() {
                ready_callbacks.push(Callback::Immediate(callback));
            }
        }

        // Process expired timers
        let now = Instant::now();
        {
            let mut timers = self.timers.lock();
            while let Some(timer) = timers.peek() {
                if timer.cancelled {
                    timers.pop();
                    continue;
                }
                if timer.deadline <= now {
                    let timer = timers.pop().unwrap();
                    ready_callbacks.push(Callback::Function(timer.callback.clone()));

                    // Reschedule if it's an interval
                    if let Some(interval) = timer.repeat {
                        let next_timer = Timer {
                            id: timer.id,
                            deadline: now + interval,
                            callback: timer.callback,
                            repeat: Some(interval),
                            cancelled: false,
                        };
                        timers.push(next_timer);
                    }
                } else {
                    break;
                }
            }
        }

        // Update pending work status
        let has_work = !self.timers.lock().is_empty()
            || !self.immediates.lock().is_empty()
            || !self.microtasks.lock().is_empty()
            || !self.promise_microtasks.lock().is_empty();
        self.has_pending_work
            .store(has_work, AtomicOrdering::SeqCst);

        // Check for unhandled rejections at end of tick
        self.check_unhandled_rejections();

        ready_callbacks
    }

    /// Check if there's pending work
    pub fn has_pending_work(&self) -> bool {
        self.has_pending_work.load(AtomicOrdering::SeqCst)
    }

    /// Get time until next timer fires (for efficient sleeping)
    pub fn time_until_next_timer(&self) -> Option<Duration> {
        let timers = self.timers.lock();
        timers.peek().map(|timer| {
            let now = Instant::now();
            if timer.deadline > now {
                timer.deadline - now
            } else {
                Duration::ZERO
            }
        })
    }

    /// Stop the event loop
    pub fn stop(&self) {
        let _ = self.event_tx.send(EventMessage::Stop);
        self.running.store(false, AtomicOrdering::SeqCst);
    }

    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(AtomicOrdering::SeqCst)
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}
