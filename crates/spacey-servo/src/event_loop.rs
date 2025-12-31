//! Event loop implementation for Servo integration.
//!
//! This module provides the event loop that manages async operations,
//! microtasks, and timers for the JavaScript runtime.

use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::Mutex;

/// A task that can be executed in the event loop.
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// The event loop for managing JavaScript async operations.
///
/// This implements the event loop model that Servo expects, handling
/// microtasks, macrotasks, and timer callbacks.
pub struct EventLoop {
    microtasks: Arc<Mutex<VecDeque<Task>>>,
    macrotasks: Arc<Mutex<VecDeque<Task>>>,
    running: Arc<Mutex<bool>>,
}

impl EventLoop {
    /// Create a new event loop.
    pub fn new() -> Self {
        Self {
            microtasks: Arc::new(Mutex::new(VecDeque::new())),
            macrotasks: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Queue a microtask (e.g., Promise resolution).
    ///
    /// Microtasks run before the next macrotask.
    pub fn queue_microtask<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.microtasks.lock().push_back(Box::new(task));
    }

    /// Queue a macrotask (e.g., setTimeout callback).
    ///
    /// Macrotasks run after all microtasks have been processed.
    pub fn queue_macrotask<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.macrotasks.lock().push_back(Box::new(task));
    }

    /// Process all pending microtasks.
    pub fn process_microtasks(&self) {
        loop {
            let task = self.microtasks.lock().pop_front();
            match task {
                Some(task) => task(),
                None => break,
            }
        }
    }

    /// Process one macrotask and all resulting microtasks.
    pub fn process_next_macrotask(&self) -> bool {
        let task = self.macrotasks.lock().pop_front();

        match task {
            Some(task) => {
                task();
                self.process_microtasks();
                true
            }
            None => false,
        }
    }

    /// Run the event loop until all tasks are complete.
    pub fn run(&self) {
        *self.running.lock() = true;

        while *self.running.lock() {
            // Process all microtasks first
            self.process_microtasks();

            // Then process one macrotask
            if !self.process_next_macrotask() {
                // No more tasks, stop the loop
                break;
            }
        }

        *self.running.lock() = false;
    }

    /// Stop the event loop.
    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    /// Check if the event loop is running.
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    /// Check if there are pending tasks.
    pub fn has_pending_tasks(&self) -> bool {
        !self.microtasks.lock().is_empty() || !self.macrotasks.lock().is_empty()
    }

    /// Get the number of pending microtasks.
    pub fn microtask_count(&self) -> usize {
        self.microtasks.lock().len()
    }

    /// Get the number of pending macrotasks.
    pub fn macrotask_count(&self) -> usize {
        self.macrotasks.lock().len()
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_create_event_loop() {
        let event_loop = EventLoop::new();
        assert!(!event_loop.is_running());
        assert!(!event_loop.has_pending_tasks());
    }

    #[test]
    fn test_queue_microtask() {
        let event_loop = EventLoop::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        event_loop.queue_microtask(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(event_loop.microtask_count(), 1);
        event_loop.process_microtasks();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(event_loop.microtask_count(), 0);
    }

    #[test]
    fn test_queue_macrotask() {
        let event_loop = EventLoop::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        event_loop.queue_macrotask(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(event_loop.macrotask_count(), 1);
        event_loop.process_next_macrotask();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(event_loop.macrotask_count(), 0);
    }

    #[test]
    fn test_microtasks_run_before_macrotasks() {
        let event_loop = EventLoop::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        let order_clone = Arc::clone(&order);
        event_loop.queue_macrotask(move || {
            order_clone.lock().push("macro1");
        });

        let order_clone = Arc::clone(&order);
        event_loop.queue_microtask(move || {
            order_clone.lock().push("micro1");
        });

        let order_clone = Arc::clone(&order);
        event_loop.queue_microtask(move || {
            order_clone.lock().push("micro2");
        });

        event_loop.run();

        let result = order.lock();
        assert_eq!(*result, vec!["micro1", "micro2", "macro1"]);
    }

    #[test]
    fn test_nested_microtasks() {
        let event_loop = EventLoop::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = Arc::clone(&counter);
        let event_loop_clone = event_loop.clone();
        event_loop.queue_microtask(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);

            let counter_clone2 = Arc::clone(&counter_clone);
            event_loop_clone.queue_microtask(move || {
                counter_clone2.fetch_add(1, Ordering::SeqCst);
            });
        });

        event_loop.process_microtasks();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}

// Implement Clone for EventLoop to support nested operations
impl Clone for EventLoop {
    fn clone(&self) -> Self {
        Self {
            microtasks: Arc::clone(&self.microtasks),
            macrotasks: Arc::clone(&self.macrotasks),
            running: Arc::clone(&self.running),
        }
    }
}
