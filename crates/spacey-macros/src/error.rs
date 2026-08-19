// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Error handling macros.
//!
//! Provides ergonomic macros for error handling patterns.

/// Early return with an error.
///
/// # Example
///
/// ```
/// use spacey_macros::bail;
///
/// fn process(x: i32) -> Result<(), String> {
///     if x < 0 {
///         bail!("x must be non-negative, got {}", x);
///     }
///     Ok(())
/// }
///
/// assert!(process(-1).is_err());
/// assert!(process(1).is_ok());
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($msg.into())
    };
    ($fmt:literal, $($arg:tt)*) => {
        return Err(format!($fmt, $($arg)*).into())
    };
    ($err:expr $(,)?) => {
        return Err($err.into())
    };
}

/// Ensure a condition is true, or return an error.
///
/// # Example
///
/// ```
/// use spacey_macros::ensure;
///
/// fn divide(a: i32, b: i32) -> Result<i32, String> {
///     ensure!(b != 0, "division by zero");
///     Ok(a / b)
/// }
///
/// assert!(divide(10, 0).is_err());
/// assert_eq!(divide(10, 2).unwrap(), 5);
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($msg.into());
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !$cond {
            return Err(format!($fmt, $($arg)*).into());
        }
    };
}

/// Unwrap an Option or return early with an error.
///
/// # Example
///
/// ```
/// use spacey_macros::try_unwrap;
///
/// fn get_value(opt: Option<i32>) -> Result<i32, String> {
///     let value = try_unwrap!(opt, "value was None");
///     Ok(value * 2)
/// }
///
/// assert_eq!(get_value(Some(5)).unwrap(), 10);
/// assert!(get_value(None).is_err());
/// ```
#[macro_export]
macro_rules! try_unwrap {
    ($opt:expr, $msg:literal) => {
        match $opt {
            Some(v) => v,
            None => return Err($msg.into()),
        }
    };
    ($opt:expr, $fmt:literal, $($arg:tt)*) => {
        match $opt {
            Some(v) => v,
            None => return Err(format!($fmt, $($arg)*).into()),
        }
    };
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err("unwrap failed on None".into()),
        }
    };
}

/// Try to execute an expression, mapping the error with a custom message.
///
/// # Example
///
/// ```
/// use spacey_macros::try_with;
///
/// fn parse_num(s: &str) -> Result<i32, String> {
///     let num = try_with!(s.parse::<i32>(), "failed to parse '{}'", s);
///     Ok(num)
/// }
///
/// assert_eq!(parse_num("42").unwrap(), 42);
/// assert!(parse_num("abc").is_err());
/// ```
#[macro_export]
macro_rules! try_with {
    ($expr:expr, $msg:literal) => {
        match $expr {
            Ok(v) => v,
            Err(_) => return Err($msg.into()),
        }
    };
    ($expr:expr, $fmt:literal, $($arg:tt)*) => {
        match $expr {
            Ok(v) => v,
            Err(_) => return Err(format!($fmt, $($arg)*).into()),
        }
    };
}

/// Return Ok(value) or continue to the next iteration.
///
/// # Example
///
/// ```
/// use spacey_macros::ok_or_continue;
///
/// let items = vec!["1", "two", "3"];
/// let mut sum = 0;
///
/// for item in items {
///     let num: i32 = ok_or_continue!(item.parse());
///     sum += num;
/// }
///
/// assert_eq!(sum, 4); // 1 + 3
/// ```
#[macro_export]
macro_rules! ok_or_continue {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(_) => continue,
        }
    };
}

/// Return Some(value) or continue to the next iteration.
///
/// # Example
///
/// ```
/// use spacey_macros::some_or_continue;
///
/// let items: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
/// let mut sum = 0;
///
/// for item in items {
///     let num = some_or_continue!(item);
///     sum += num;
/// }
///
/// assert_eq!(sum, 4);
/// ```
#[macro_export]
macro_rules! some_or_continue {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => continue,
        }
    };
}

/// Return Ok(value) or break from the loop.
///
/// # Example
///
/// ```
/// use spacey_macros::ok_or_break;
///
/// let items = vec!["1", "2", "three", "4"];
/// let mut values = Vec::new();
///
/// for item in items {
///     let num: i32 = ok_or_break!(item.parse());
///     values.push(num);
/// }
///
/// assert_eq!(values, vec![1, 2]); // Stopped at "three"
/// ```
#[macro_export]
macro_rules! ok_or_break {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(_) => break,
        }
    };
}

/// Create a custom error type quickly.
///
/// # Example
///
/// ```
/// use spacey_macros::error_type;
///
/// error_type!(ParseError, "failed to parse input");
///
/// fn parse(s: &str) -> Result<i32, ParseError> {
///     s.parse().map_err(|_| ParseError)
/// }
///
/// assert!(parse("abc").is_err());
/// ```
#[macro_export]
macro_rules! error_type {
    ($name:ident, $msg:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, $msg)
            }
        }

        impl ::std::error::Error for $name {}
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ensure() {
        fn check(x: i32) -> Result<(), String> {
            ensure!(x > 0, "must be positive");
            Ok(())
        }

        assert!(check(1).is_ok());
        assert!(check(0).is_err());
    }

    #[test]
    fn test_bail() {
        fn early_return(fail: bool) -> Result<i32, String> {
            if fail {
                bail!("failed");
            }
            Ok(42)
        }

        assert_eq!(early_return(false).unwrap(), 42);
        assert!(early_return(true).is_err());
    }

    #[test]
    fn test_try_unwrap() {
        fn get(opt: Option<i32>) -> Result<i32, String> {
            let v = try_unwrap!(opt, "none");
            Ok(v)
        }

        assert_eq!(get(Some(5)).unwrap(), 5);
        assert!(get(None).is_err());
    }

    #[test]
    fn test_try_with() {
        fn parse(s: &str) -> Result<i32, String> {
            let num = try_with!(s.parse::<i32>(), "invalid number");
            Ok(num)
        }

        assert_eq!(parse("42").unwrap(), 42);
        assert!(parse("abc").is_err());
    }

    #[test]
    fn test_error_type() {
        error_type!(TestError, "test error occurred");

        let err = TestError;
        assert_eq!(format!("{}", err), "test error occurred");
    }
}
