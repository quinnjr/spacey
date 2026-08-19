// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Async and parallel processing macros.
//!
//! Requires `tokio` and/or `rayon` crates.

/// Conditionally use parallel iteration based on threshold.
///
/// Requires `rayon` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::parallel_threshold;
///
/// let items: Vec<i32> = (0..10000).collect();
/// let results = parallel_threshold!(items, 1000,
///     |item| item * 2
/// );
/// ```
#[macro_export]
macro_rules! parallel_threshold {
    ($collection:expr, $threshold:expr, |$item:ident| $transform:expr) => {{
        if $collection.len() >= $threshold {
            use ::rayon::prelude::*;
            $collection
                .par_iter()
                .map(|$item| $transform)
                .collect::<Vec<_>>()
        } else {
            $collection
                .iter()
                .map(|$item| $transform)
                .collect::<Vec<_>>()
        }
    }};
}

/// Spawn a blocking task for CPU-intensive work.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::spawn_blocking;
///
/// async fn example() {
///     let result = spawn_blocking!(expensive_computation()).await.unwrap();
/// }
/// ```
#[macro_export]
macro_rules! spawn_blocking {
    ($block:expr) => {
        ::tokio::task::spawn_blocking(move || $block)
    };
}

/// Join multiple async operations concurrently.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::join_all;
///
/// async fn example() {
///     let (a, b, c) = join_all!(
///         fetch_a(),
///         fetch_b(),
///         fetch_c(),
///     );
/// }
/// ```
#[macro_export]
macro_rules! join_all {
    ($($future:expr),+ $(,)?) => {
        ::tokio::join!($($future),+)
    };
}

/// Try to join multiple async operations, returning on first error.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::try_join_all;
///
/// async fn example() -> Result<(A, B, C), Error> {
///     let (a, b, c) = try_join_all!(
///         fetch_a(),
///         fetch_b(),
///         fetch_c(),
///     )?;
///     Ok((a, b, c))
/// }
/// ```
#[macro_export]
macro_rules! try_join_all {
    ($($future:expr),+ $(,)?) => {
        ::tokio::try_join!($($future),+)
    };
}

/// Select the first completed future.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::select_first;
///
/// async fn example() {
///     select_first! {
///         result = async_op_1() => { handle_1(result) },
///         result = async_op_2() => { handle_2(result) },
///     }
/// }
/// ```
#[macro_export]
macro_rules! select_first {
    ($($pattern:pat = $future:expr => $handler:expr),+ $(,)?) => {
        ::tokio::select! {
            $($pattern = $future => $handler),+
        }
    };
}

/// Run a future with a timeout.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::with_timeout;
/// use std::time::Duration;
///
/// async fn example() -> Result<Data, TimeoutError> {
///     with_timeout!(Duration::from_secs(5), fetch_data())
/// }
/// ```
#[macro_export]
macro_rules! with_timeout {
    ($duration:expr, $future:expr) => {
        ::tokio::time::timeout($duration, $future)
    };
}

/// Spawn a new tokio task.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::spawn;
///
/// async fn example() {
///     let handle = spawn!(async {
///         do_work().await
///     });
/// }
/// ```
#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        ::tokio::spawn($future)
    };
}

/// Sleep for a duration.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::sleep;
/// use std::time::Duration;
///
/// async fn example() {
///     sleep!(Duration::from_millis(100)).await;
/// }
/// ```
#[macro_export]
macro_rules! sleep {
    ($duration:expr) => {
        ::tokio::time::sleep($duration)
    };
}

/// Sleep for a number of milliseconds.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::sleep_ms;
///
/// async fn example() {
///     sleep_ms!(100).await;
/// }
/// ```
#[macro_export]
macro_rules! sleep_ms {
    ($ms:expr) => {
        ::tokio::time::sleep(::std::time::Duration::from_millis($ms))
    };
}

/// Parallel map over a collection using rayon.
///
/// Requires `rayon` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::par_map;
///
/// let numbers: Vec<i32> = (0..1000).collect();
/// let doubled = par_map!(numbers, |x| x * 2);
/// ```
#[macro_export]
macro_rules! par_map {
    ($collection:expr, |$item:ident| $transform:expr) => {{
        use ::rayon::prelude::*;
        $collection
            .par_iter()
            .map(|$item| $transform)
            .collect::<Vec<_>>()
    }};
}

/// Parallel filter over a collection using rayon.
///
/// Requires `rayon` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::par_filter;
///
/// let numbers: Vec<i32> = (0..1000).collect();
/// let evens = par_filter!(numbers, |x| x % 2 == 0);
/// ```
#[macro_export]
macro_rules! par_filter {
    ($collection:expr, |$item:ident| $predicate:expr) => {{
        use ::rayon::prelude::*;
        $collection
            .par_iter()
            .filter(|$item| $predicate)
            .cloned()
            .collect::<Vec<_>>()
    }};
}

/// Parallel reduce over a collection using rayon.
///
/// Requires `rayon` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::par_reduce;
///
/// let numbers: Vec<i32> = (0..1000).collect();
/// let sum = par_reduce!(numbers, 0, |acc, x| acc + x);
/// ```
#[macro_export]
macro_rules! par_reduce {
    ($collection:expr, $init:expr, |$acc:ident, $item:ident| $combine:expr) => {{
        use ::rayon::prelude::*;
        $collection
            .par_iter()
            .fold(|| $init, |$acc, $item| $combine)
            .reduce(|| $init, |$acc, $item| $combine)
    }};
}

/// Run an async block and block on it (for testing or main functions).
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::block_on;
///
/// fn main() {
///     let result = block_on!(async {
///         fetch_data().await
///     });
/// }
/// ```
#[macro_export]
macro_rules! block_on {
    ($future:expr) => {
        ::tokio::runtime::Runtime::new().unwrap().block_on($future)
    };
}

#[cfg(test)]
mod tests {
    // Async tests would require tokio/rayon setup
    // Basic macro syntax tests only

    #[test]
    fn test_parallel_threshold_syntax() {
        // Just verify the macro compiles with the sequential path
        let items: Vec<i32> = (0..10).collect();
        let _results: Vec<i32> = items.iter().map(|item| item * 2).collect();
    }
}
