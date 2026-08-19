// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Control flow macros.
//!
//! Provides macros for scope guards, retries, and control flow patterns.

/// Defer execution until scope exit (like Go's defer).
///
/// # Example
///
/// ```
/// use spacey_macros::defer;
///
/// fn example() {
///     defer!(println!("cleanup"));
///     println!("work");
///     // prints: work, then cleanup
/// }
/// ```
#[macro_export]
macro_rules! defer {
    ($($body:tt)*) => {
        let _guard = $crate::DeferGuard(Some(|| { $($body)* }));
    };
}

/// Retry an operation with exponential backoff.
///
/// # Example
///
/// ```
/// use spacey_macros::retry;
///
/// fn fallible() -> Result<i32, &'static str> {
///     static mut ATTEMPTS: i32 = 0;
///     unsafe {
///         ATTEMPTS += 1;
///         if ATTEMPTS < 3 {
///             Err("not yet")
///         } else {
///             Ok(42)
///         }
///     }
/// }
///
/// let result = retry!(5, fallible());
/// assert!(result.is_ok());
/// ```
#[macro_export]
macro_rules! retry {
    ($max_attempts:expr, $op:expr) => {{
        let mut attempts = 0;
        loop {
            attempts += 1;
            match $op {
                Ok(v) => break Ok(v),
                Err(e) if attempts >= $max_attempts => break Err(e),
                Err(_) => {
                    ::std::thread::sleep(::std::time::Duration::from_millis(
                        10 * (1 << attempts.min(6)),
                    ));
                }
            }
        }
    }};
}

/// Retry with custom delay between attempts.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::retry_with_delay;
/// use std::time::Duration;
///
/// let result = retry_with_delay!(3, Duration::from_millis(100), fetch_data());
/// ```
#[macro_export]
macro_rules! retry_with_delay {
    ($max_attempts:expr, $delay:expr, $op:expr) => {{
        let mut attempts = 0;
        let delay = $delay;
        loop {
            attempts += 1;
            match $op {
                Ok(v) => break Ok(v),
                Err(e) if attempts >= $max_attempts => break Err(e),
                Err(_) => {
                    ::std::thread::sleep(delay);
                }
            }
        }
    }};
}

/// Execute a block if a condition is true, returns Option.
///
/// # Example
///
/// ```
/// use spacey_macros::when;
///
/// let x = 5;
/// let result = when!(x > 3, x * 2);
/// assert_eq!(result, Some(10));
///
/// let result2 = when!(x < 3, x * 2);
/// assert_eq!(result2, None);
/// ```
#[macro_export]
macro_rules! when {
    ($cond:expr, $then:expr) => {
        if $cond { Some($then) } else { None }
    };
}

/// Execute one of two blocks based on a condition.
///
/// # Example
///
/// ```
/// use spacey_macros::if_else;
///
/// let x = 5;
/// let result = if_else!(x > 3, "big", "small");
/// assert_eq!(result, "big");
/// ```
#[macro_export]
macro_rules! if_else {
    ($cond:expr, $then:expr, $else:expr) => {
        if $cond { $then } else { $else }
    };
}

/// Loop until a condition becomes true, with optional timeout.
///
/// # Example
///
/// ```
/// use spacey_macros::loop_until;
///
/// let mut counter = 0;
/// loop_until!(counter >= 5, {
///     counter += 1;
/// });
/// assert_eq!(counter, 5);
/// ```
#[macro_export]
macro_rules! loop_until {
    ($cond:expr, $body:block) => {
        while !$cond {
            $body
        }
    };
}

/// Execute a block a specific number of times.
///
/// # Example
///
/// ```
/// use spacey_macros::times;
///
/// let mut sum = 0;
/// times!(5, {
///     sum += 1;
/// });
/// assert_eq!(sum, 5);
/// ```
#[macro_export]
macro_rules! times {
    ($n:expr, $body:block) => {
        for _ in 0..$n {
            $body
        }
    };
}

/// Execute a block a specific number of times with index.
///
/// # Example
///
/// ```
/// use spacey_macros::times_with_index;
///
/// let mut sum = 0;
/// times_with_index!(i, 5, {
///     sum += i;
/// });
/// assert_eq!(sum, 10); // 0 + 1 + 2 + 3 + 4
/// ```
#[macro_export]
macro_rules! times_with_index {
    ($idx:ident, $n:expr, $body:block) => {
        for $idx in 0..$n {
            $body
        }
    };
}

/// Guard clause - early return if condition is true.
///
/// # Example
///
/// ```
/// use spacey_macros::guard;
///
/// fn process(x: i32) -> Option<i32> {
///     guard!(x < 0, None);
///     Some(x * 2)
/// }
///
/// assert_eq!(process(-1), None);
/// assert_eq!(process(5), Some(10));
/// ```
#[macro_export]
macro_rules! guard {
    ($cond:expr, $ret:expr) => {
        if $cond {
            return $ret;
        }
    };
}

/// Guard clause with negation - early return if condition is false.
///
/// # Example
///
/// ```
/// use spacey_macros::guard_let;
///
/// fn process(x: Option<i32>) -> i32 {
///     guard_let!(x.is_some(), 0);
///     x.unwrap() * 2
/// }
///
/// assert_eq!(process(None), 0);
/// assert_eq!(process(Some(5)), 10);
/// ```
#[macro_export]
macro_rules! guard_let {
    ($cond:expr, $ret:expr) => {
        if !$cond {
            return $ret;
        }
    };
}

/// Run cleanup on scope exit, even on panic.
///
/// # Example
///
/// ```
/// use spacey_macros::scope_exit;
/// use std::cell::RefCell;
///
/// let log = RefCell::new(Vec::new());
///
/// {
///     log.borrow_mut().push("start");
///     scope_exit!(log.borrow_mut().push("cleanup"));
///     log.borrow_mut().push("work");
/// }
///
/// assert_eq!(*log.borrow(), vec!["start", "work", "cleanup"]);
/// ```
#[macro_export]
macro_rules! scope_exit {
    ($($body:tt)*) => {
        let _guard = $crate::DeferGuard(Some(|| { $($body)* }));
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    #[test]
    fn test_defer() {
        let order = RefCell::new(Vec::new());

        {
            order.borrow_mut().push(1);
            defer!(order.borrow_mut().push(3));
            order.borrow_mut().push(2);
        }

        assert_eq!(*order.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_when() {
        assert_eq!(when!(true, 42), Some(42));
        assert_eq!(when!(false, 42), None);
    }

    #[test]
    fn test_if_else() {
        assert_eq!(if_else!(true, "yes", "no"), "yes");
        assert_eq!(if_else!(false, "yes", "no"), "no");
    }

    #[test]
    fn test_times() {
        let mut count = 0;
        times!(3, {
            count += 1;
        });
        assert_eq!(count, 3);
    }

    #[test]
    fn test_times_with_index() {
        let mut sum = 0;
        times_with_index!(i, 5, {
            sum += i;
        });
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_guard() {
        fn check(x: i32) -> i32 {
            guard!(x < 0, -1);
            x * 2
        }

        assert_eq!(check(-5), -1);
        assert_eq!(check(5), 10);
    }
}
