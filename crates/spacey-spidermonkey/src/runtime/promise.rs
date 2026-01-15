//! Promise implementation.

use std::collections::VecDeque;

/// The state of a Promise.
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    /// Promise is pending.
    Pending,
    /// Promise has been fulfilled with a value.
    Fulfilled(PromiseValue),
    /// Promise has been rejected with a reason.
    Rejected(PromiseValue),
}

/// A value stored in a Promise (simplified for now).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PromiseValue {
    /// Undefined
    #[default]
    Undefined,
    /// Null
    Null,
    /// Boolean
    Boolean(bool),
    /// Number
    Number(f64),
    /// String
    String(String),
    /// Object reference (would be a GC handle in full implementation)
    Object(usize),
}

/// A Promise reaction (callback to be called when Promise settles).
#[derive(Debug, Clone)]
pub struct PromiseReaction {
    /// The fulfill handler
    pub on_fulfilled: Option<usize>, // Function reference
    /// The reject handler
    pub on_rejected: Option<usize>, // Function reference
    /// The promise to resolve with the handler's result
    pub result_promise: Option<usize>, // Promise reference
}

/// A Promise object.
#[derive(Debug, Clone)]
pub struct Promise {
    /// The current state
    pub state: PromiseState,
    /// Fulfill reactions (callbacks for .then())
    pub fulfill_reactions: Vec<PromiseReaction>,
    /// Reject reactions (callbacks for .catch())
    pub reject_reactions: Vec<PromiseReaction>,
    /// Whether this promise is being handled
    pub is_handled: bool,
}

impl Promise {
    /// Creates a new pending Promise.
    pub fn new() -> Self {
        Self {
            state: PromiseState::Pending,
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
            is_handled: false,
        }
    }

    /// Creates a Promise that is already fulfilled with a value.
    pub fn resolved(value: PromiseValue) -> Self {
        Self {
            state: PromiseState::Fulfilled(value),
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
            is_handled: false,
        }
    }

    /// Creates a Promise that is already rejected with a reason.
    pub fn rejected(reason: PromiseValue) -> Self {
        Self {
            state: PromiseState::Rejected(reason),
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
            is_handled: false,
        }
    }

    /// Returns true if this promise is pending.
    pub fn is_pending(&self) -> bool {
        matches!(self.state, PromiseState::Pending)
    }

    /// Returns true if this promise is fulfilled.
    pub fn is_fulfilled(&self) -> bool {
        matches!(self.state, PromiseState::Fulfilled(_))
    }

    /// Returns true if this promise is rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self.state, PromiseState::Rejected(_))
    }
}

impl Default for Promise {
    fn default() -> Self {
        Self::new()
    }
}

/// A microtask queue for scheduling Promise reactions.
#[derive(Debug, Default)]
pub struct MicrotaskQueue {
    /// Queue of microtasks to execute
    tasks: VecDeque<Microtask>,
}

/// A microtask to execute.
#[derive(Debug, Clone)]
pub enum Microtask {
    /// Execute a promise reaction
    PromiseReaction {
        /// The reaction to execute
        reaction: PromiseReaction,
        /// The value to pass to the handler
        value: PromiseValue,
        /// Whether this is a fulfill or reject reaction
        is_fulfill: bool,
    },
    /// Execute a resolve thenable job
    ResolveThenable {
        /// The promise to resolve
        promise_to_resolve: usize,
        /// The thenable to resolve with
        thenable: usize,
        /// The then function
        then_fn: usize,
    },
}

impl MicrotaskQueue {
    /// Creates a new empty microtask queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues a microtask.
    pub fn enqueue(&mut self, task: Microtask) {
        self.tasks.push_back(task);
    }

    /// Dequeues the next microtask, if any.
    pub fn dequeue(&mut self) -> Option<Microtask> {
        self.tasks.pop_front()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the number of pending microtasks.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_new() {
        let promise = Promise::new();
        assert!(promise.is_pending());
        assert!(!promise.is_fulfilled());
        assert!(!promise.is_rejected());
    }

    #[test]
    fn test_promise_resolved() {
        let promise = Promise::resolved(PromiseValue::Number(42.0));
        assert!(!promise.is_pending());
        assert!(promise.is_fulfilled());
        assert!(!promise.is_rejected());
    }

    #[test]
    fn test_promise_rejected() {
        let promise = Promise::rejected(PromiseValue::String("error".to_string()));
        assert!(!promise.is_pending());
        assert!(!promise.is_fulfilled());
        assert!(promise.is_rejected());
    }

    #[test]
    fn test_microtask_queue() {
        let mut queue = MicrotaskQueue::new();
        assert!(queue.is_empty());

        queue.enqueue(Microtask::PromiseReaction {
            reaction: PromiseReaction {
                on_fulfilled: None,
                on_rejected: None,
                result_promise: None,
            },
            value: PromiseValue::Undefined,
            is_fulfill: true,
        });

        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);

        let task = queue.dequeue();
        assert!(task.is_some());
        assert!(queue.is_empty());
    }
}
