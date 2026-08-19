// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Testing helper macros.
//!
//! Provides macros for writing cleaner and more expressive tests.

/// Assert that an expression matches a pattern.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_matches;
///
/// #[derive(Debug)]
/// enum MyResult { Ok(i32), Err(String) }
///
/// let result = MyResult::Ok(42);
/// assert_matches!(result, MyResult::Ok(n) if n > 0);
/// ```
#[macro_export]
macro_rules! assert_matches {
    ($expr:expr, $pat:pat) => {
        match $expr {
            $pat => {}
            ref e => panic!(
                "assertion failed: `{}` does not match pattern `{}`\n  value: {:?}",
                stringify!($expr),
                stringify!($pat),
                e
            ),
        }
    };
    ($expr:expr, $pat:pat if $guard:expr) => {
        match $expr {
            $pat if $guard => {}
            ref e => panic!(
                "assertion failed: `{}` does not match pattern `{} if {}`\n  value: {:?}",
                stringify!($expr),
                stringify!($pat),
                stringify!($guard),
                e
            ),
        }
    };
}

/// Assert that a Result is Ok and extract the value.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_ok;
///
/// fn divide(a: i32, b: i32) -> Result<i32, String> {
///     if b == 0 { Err("division by zero".into()) } else { Ok(a / b) }
/// }
///
/// let value = assert_ok!(divide(10, 2));
/// assert_eq!(value, 5);
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!(
                "assertion failed: expected Ok, got Err\n  expression: `{}`\n  error: {:?}",
                stringify!($expr),
                e
            ),
        }
    };
}

/// Assert that a Result is Err.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_err;
///
/// fn divide(a: i32, b: i32) -> Result<i32, String> {
///     if b == 0 { Err("division by zero".into()) } else { Ok(a / b) }
/// }
///
/// assert_err!(divide(10, 0));
/// ```
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(v) => panic!(
                "assertion failed: expected Err, got Ok\n  expression: `{}`\n  value: {:?}",
                stringify!($expr),
                v
            ),
            Err(_) => {}
        }
    };
}

/// Assert that an Option is Some and extract the value.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_some;
///
/// let opt = Some(42);
/// let value = assert_some!(opt);
/// assert_eq!(value, 42);
/// ```
#[macro_export]
macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => panic!(
                "assertion failed: expected Some, got None\n  expression: `{}`",
                stringify!($expr)
            ),
        }
    };
}

/// Assert that an Option is None.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_none;
///
/// let opt: Option<i32> = None;
/// assert_none!(opt);
/// ```
#[macro_export]
macro_rules! assert_none {
    ($expr:expr) => {
        match $expr {
            Some(v) => panic!(
                "assertion failed: expected None, got Some\n  expression: `{}`\n  value: {:?}",
                stringify!($expr),
                v
            ),
            None => {}
        }
    };
}

/// Assert that two floating point numbers are approximately equal.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_approx_eq;
///
/// assert_approx_eq!(0.1 + 0.2, 0.3, 1e-10);
/// ```
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left = $left as f64;
        let right = $right as f64;
        let epsilon = $epsilon as f64;
        let diff = (left - right).abs();
        if diff > epsilon {
            panic!(
                "assertion failed: `(left ≈ right)`\n  left: `{}`\n  right: `{}`\n  diff: `{}` > epsilon `{}`",
                left, right, diff, epsilon
            );
        }
    }};
}

/// Assert that a string contains a substring.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_contains;
///
/// let s = "hello world";
/// assert_contains!(s, "world");
/// ```
#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        if !$haystack.contains($needle) {
            panic!(
                "assertion failed: string does not contain substring\n  string: `{}`\n  expected: `{}`",
                $haystack, $needle
            );
        }
    };
}

/// Assert that a collection has a specific length.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_len;
///
/// let v = vec![1, 2, 3];
/// assert_len!(v, 3);
/// ```
#[macro_export]
macro_rules! assert_len {
    ($collection:expr, $expected:expr) => {{
        let actual = $collection.len();
        if actual != $expected {
            panic!(
                "assertion failed: length mismatch\n  expected: `{}`\n  actual: `{}`",
                $expected, actual
            );
        }
    }};
}

/// Assert that a collection is empty.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_empty;
///
/// let v: Vec<i32> = vec![];
/// assert_empty!(v);
/// ```
#[macro_export]
macro_rules! assert_empty {
    ($collection:expr) => {
        if !$collection.is_empty() {
            panic!(
                "assertion failed: collection is not empty\n  length: `{}`",
                $collection.len()
            );
        }
    };
}

/// Assert that a collection is not empty.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_not_empty;
///
/// let v = vec![1, 2, 3];
/// assert_not_empty!(v);
/// ```
#[macro_export]
macro_rules! assert_not_empty {
    ($collection:expr) => {
        if $collection.is_empty() {
            panic!("assertion failed: collection is empty");
        }
    };
}

/// Create a test case with setup and teardown.
///
/// # Example
///
/// ```
/// use spacey_macros::test_case;
///
/// test_case!(test_example,
///     setup: { let data = vec![1, 2, 3]; },
///     test: { assert_eq!(data.len(), 3); },
///     teardown: { /* cleanup */ }
/// );
/// ```
#[macro_export]
macro_rules! test_case {
    (
        $name:ident,
        setup: { $($setup:tt)* },
        test: { $($test:tt)* }
        $(, teardown: { $($teardown:tt)* })?
    ) => {
        #[test]
        fn $name() {
            $($setup)*
            let _result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                $($test)*
            }));
            $(
                $($teardown)*
            )?
            if let Err(e) = _result {
                ::std::panic::resume_unwind(e);
            }
        }
    };
}

/// Create multiple parameterized test cases.
///
/// # Example
///
/// ```
/// use spacey_macros::parameterized_test;
///
/// parameterized_test!(
///     test_double,
///     (input, expected),
///     [
///         (1, 2),
///         (2, 4),
///         (5, 10),
///     ],
///     {
///         assert_eq!(input * 2, expected);
///     }
/// );
/// ```
#[macro_export]
macro_rules! parameterized_test {
    (
        $name:ident,
        ($($param:ident),+),
        [$(($($value:expr),+)),+ $(,)?],
        $body:block
    ) => {
        #[test]
        fn $name() {
            let test_cases = vec![
                $(($($value),+)),+
            ];

            for (i, ($($param),+)) in test_cases.into_iter().enumerate() {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| $body));
                if result.is_err() {
                    panic!("Test case {} failed with params: {:?}", i, ($(stringify!($param)),+));
                }
            }
        }
    };
}

/// Skip a test with a reason.
///
/// # Example
///
/// ```
/// use spacey_macros::skip_test;
///
/// #[test]
/// fn test_feature() {
///     skip_test!("Feature not implemented yet");
/// }
/// ```
#[macro_export]
macro_rules! skip_test {
    ($reason:literal) => {
        eprintln!("SKIPPED: {}", $reason);
        return;
    };
}

/// Assert that a block panics with a specific message.
///
/// # Example
///
/// ```
/// use spacey_macros::assert_panics;
///
/// assert_panics!({ panic!("expected panic"); }, "expected panic");
/// ```
#[macro_export]
macro_rules! assert_panics {
    ($block:block, $msg:expr) => {{
        let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| $block));
        match result {
            Ok(_) => panic!("assertion failed: expected panic, but block completed normally"),
            Err(e) => {
                let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    String::from("unknown panic")
                };
                if !panic_msg.contains($msg) {
                    panic!(
                        "assertion failed: panic message mismatch\n  expected: `{}`\n  actual: `{}`",
                        $msg, panic_msg
                    );
                }
            }
        }
    }};
}

/// Create a mock function that records calls.
///
/// # Example
///
/// ```
/// use spacey_macros::mock_fn;
/// use std::cell::RefCell;
///
/// let calls: RefCell<Vec<i32>> = RefCell::new(Vec::new());
/// let mock = mock_fn!(|x: i32| {
///     calls.borrow_mut().push(x);
///     x * 2
/// });
///
/// assert_eq!(mock(5), 10);
/// assert_eq!(*calls.borrow(), vec![5]);
/// ```
#[macro_export]
macro_rules! mock_fn {
    (|$($arg:ident : $type:ty),*| $body:expr) => {
        |$($arg: $type),*| $body
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assert_matches() {
        #[derive(Debug)]
        enum E {
            A(i32),
            B,
        }
        assert_matches!(E::A(42), E::A(_));
        assert_matches!(E::A(42), E::A(n) if n > 0);
    }

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok!(result);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        assert_err!(result);
    }

    #[test]
    fn test_assert_some() {
        let opt = Some(42);
        let value = assert_some!(opt);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_none() {
        let opt: Option<i32> = None;
        assert_none!(opt);
    }

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq!(0.1 + 0.2, 0.3, 1e-10);
        assert_approx_eq!(1.0, 1.0001, 0.001);
    }

    #[test]
    fn test_assert_contains() {
        assert_contains!("hello world", "world");
        assert_contains!("hello world", "hello");
    }

    #[test]
    fn test_assert_len() {
        assert_len!(vec![1, 2, 3], 3);
        assert_len!("hello", 5);
    }

    #[test]
    fn test_assert_empty() {
        let v: Vec<i32> = vec![];
        assert_empty!(v);
    }

    #[test]
    fn test_assert_not_empty() {
        let v = vec![1];
        assert_not_empty!(v);
    }
}
