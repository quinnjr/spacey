// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Debug and development macros.
//!
//! Provides macros for debugging, timing, and development workflows.

/// Measure and print execution time of a block.
///
/// # Example
///
/// ```
/// use spacey_macros::time_it;
///
/// let result = time_it!("computation", {
///     let mut sum = 0;
///     for i in 0..100 {
///         sum += i;
///     }
///     sum
/// });
/// assert_eq!(result, 4950);
/// ```
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:expr) => {{
        let __start = ::std::time::Instant::now();
        let __result = $block;
        let __elapsed = __start.elapsed();
        eprintln!("[TIMER] {} took {:?}", $name, __elapsed);
        __result
    }};
}

/// Measure execution time and return both duration and result.
///
/// # Example
///
/// ```
/// use spacey_macros::timed;
///
/// let (duration, result) = timed!({
///     (0..100).sum::<i32>()
/// });
///
/// assert_eq!(result, 4950);
/// assert!(duration.as_nanos() > 0);
/// ```
#[macro_export]
macro_rules! timed {
    ($block:expr) => {{
        let __start = ::std::time::Instant::now();
        let __result = $block;
        let __elapsed = __start.elapsed();
        (__elapsed, __result)
    }};
}

/// Debug log that compiles out in release builds.
///
/// # Example
///
/// ```
/// use spacey_macros::debug_log;
///
/// debug_log!("processing item: {}", 42);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] {}", format!($($arg)*));
    };
}

/// Trace log with file and line information (debug builds only).
///
/// # Example
///
/// ```
/// use spacey_macros::trace_log;
///
/// trace_log!("entering function");
/// ```
#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[TRACE {}:{}] {}", file!(), line!(), format!($($arg)*));
    };
}

/// Print a debug representation of a value with its name.
///
/// # Example
///
/// ```
/// use spacey_macros::dbg_named;
///
/// let x = 42;
/// dbg_named!(x); // Prints: x = 42
/// ```
#[macro_export]
macro_rules! dbg_named {
    ($val:expr) => {
        #[cfg(debug_assertions)]
        eprintln!("{} = {:?}", stringify!($val), $val);
        #[cfg(not(debug_assertions))]
        {
            let _ = &$val;
        }
    };
}

/// Print memory size of a type.
///
/// # Example
///
/// ```
/// use spacey_macros::sizeof;
///
/// sizeof!(u64);  // Prints: size_of::<u64> = 8 bytes
/// sizeof!(String);
/// ```
#[macro_export]
macro_rules! sizeof {
    ($t:ty) => {
        eprintln!(
            "size_of::<{}> = {} bytes",
            stringify!($t),
            ::std::mem::size_of::<$t>()
        );
    };
}

/// Print alignment of a type.
///
/// # Example
///
/// ```
/// use spacey_macros::alignof;
///
/// alignof!(u64);  // Prints: align_of::<u64> = 8 bytes
/// ```
#[macro_export]
macro_rules! alignof {
    ($t:ty) => {
        eprintln!(
            "align_of::<{}> = {} bytes",
            stringify!($t),
            ::std::mem::align_of::<$t>()
        );
    };
}

/// Benchmark a block multiple times and print statistics.
///
/// # Example
///
/// ```
/// use spacey_macros::bench;
///
/// bench!("sum", 100, {
///     (0..1000).sum::<i32>()
/// });
/// ```
#[macro_export]
macro_rules! bench {
    ($name:expr, $iterations:expr, $block:expr) => {{
        let mut times = Vec::with_capacity($iterations);
        let mut result = None;

        for _ in 0..$iterations {
            let start = ::std::time::Instant::now();
            result = Some($block);
            times.push(start.elapsed());
        }

        let total: ::std::time::Duration = times.iter().sum();
        let avg = total / $iterations as u32;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();

        eprintln!(
            "[BENCH] {} ({} iterations): avg={:?}, min={:?}, max={:?}",
            $name, $iterations, avg, min, max
        );

        result.unwrap()
    }};
}

/// Log entry and exit of a function (debug builds only).
///
/// # Example
///
/// ```
/// use spacey_macros::fn_trace;
///
/// fn my_function() {
///     fn_trace!();
///     // function body
/// }
/// ```
#[macro_export]
macro_rules! fn_trace {
    () => {
        #[cfg(debug_assertions)]
        {
            eprintln!("[ENTER] {}:{} {}", file!(), line!(), module_path!());
            let _guard = $crate::DeferGuard(Some(|| {
                eprintln!("[EXIT] {}:{} {}", file!(), line!(), module_path!());
            }));
        }
    };
}

/// Assert in debug mode, no-op in release.
///
/// # Example
///
/// ```
/// use spacey_macros::debug_assert_eq;
///
/// debug_assert_eq!(2 + 2, 4);
/// ```
#[macro_export]
macro_rules! debug_assert_eq {
    ($left:expr, $right:expr) => {
        #[cfg(debug_assertions)]
        assert_eq!($left, $right);
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        assert_eq!($left, $right, $($arg)+);
    };
}

/// Print the type of an expression.
///
/// # Example
///
/// ```
/// use spacey_macros::print_type;
///
/// let x = vec![1, 2, 3];
/// print_type!(x);  // Prints: type of `x` = alloc::vec::Vec<i32>
/// ```
#[macro_export]
macro_rules! print_type {
    ($val:expr) => {{
        fn type_name_of<T>(_: &T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        eprintln!("type of `{}` = {}", stringify!($val), type_name_of(&$val));
    }};
}

/// Conditional compilation block that only runs in debug mode.
///
/// # Example
///
/// ```
/// use spacey_macros::debug_only;
///
/// debug_only! {
///     eprintln!("This only runs in debug builds");
/// }
/// ```
#[macro_export]
macro_rules! debug_only {
    ($($body:tt)*) => {
        #[cfg(debug_assertions)]
        {
            $($body)*
        }
    };
}

/// Conditional compilation block that only runs in release mode.
///
/// # Example
///
/// ```
/// use spacey_macros::release_only;
///
/// release_only! {
///     // optimized code path
/// }
/// ```
#[macro_export]
macro_rules! release_only {
    ($($body:tt)*) => {
        #[cfg(not(debug_assertions))]
        {
            $($body)*
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_timed() {
        let (duration, result) = timed!({
            let mut sum = 0;
            for i in 0..100 {
                sum += i;
            }
            sum
        });

        assert_eq!(result, 4950);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_bench() {
        let result = bench!("test_sum", 10, { (0..100).sum::<i32>() });
        assert_eq!(result, 4950);
    }
}
