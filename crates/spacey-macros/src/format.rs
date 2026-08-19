// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Formatting and conversion macros.
//!
//! Provides macros for formatting values and common conversions.

/// Format bytes as human-readable size.
///
/// # Example
///
/// ```
/// use spacey_macros::format_bytes;
///
/// assert_eq!(format_bytes!(1024), "1.00 KB");
/// assert_eq!(format_bytes!(1024 * 1024), "1.00 MB");
/// assert_eq!(format_bytes!(500), "500 B");
/// ```
#[macro_export]
macro_rules! format_bytes {
    ($bytes:expr) => {{
        let bytes = $bytes as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.2} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.2} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else {
            format!("{} B", bytes as u64)
        }
    }};
}

/// Clamp a value between min and max.
///
/// # Example
///
/// ```
/// use spacey_macros::clamp;
///
/// assert_eq!(clamp!(5, 0, 10), 5);
/// assert_eq!(clamp!(-5, 0, 10), 0);
/// assert_eq!(clamp!(15, 0, 10), 10);
/// ```
#[macro_export]
macro_rules! clamp {
    ($value:expr, $min:expr, $max:expr) => {{
        let v = $value;
        let min = $min;
        let max = $max;
        if v < min {
            min
        } else if v > max {
            max
        } else {
            v
        }
    }};
}

/// Format a duration in a human-readable way.
///
/// # Example
///
/// ```
/// use spacey_macros::format_duration;
/// use std::time::Duration;
///
/// assert_eq!(format_duration!(Duration::from_secs(90)), "1m 30s");
/// assert_eq!(format_duration!(Duration::from_millis(1500)), "1.50s");
/// assert_eq!(format_duration!(Duration::from_micros(500)), "500µs");
/// ```
#[macro_export]
macro_rules! format_duration {
    ($duration:expr) => {{
        let d: ::std::time::Duration = $duration;
        let secs = d.as_secs();
        let millis = d.as_millis();
        let micros = d.as_micros();
        let nanos = d.as_nanos();

        if secs >= 60 {
            let mins = secs / 60;
            let rem_secs = secs % 60;
            if rem_secs > 0 {
                format!("{}m {}s", mins, rem_secs)
            } else {
                format!("{}m", mins)
            }
        } else if secs > 0 {
            let frac = (millis % 1000) as f64 / 1000.0;
            if frac > 0.0 {
                format!("{:.2}s", secs as f64 + frac)
            } else {
                format!("{}s", secs)
            }
        } else if millis > 0 {
            format!("{}ms", millis)
        } else if micros > 0 {
            format!("{}µs", micros)
        } else {
            format!("{}ns", nanos)
        }
    }};
}

/// Format a number with thousands separators.
///
/// # Example
///
/// ```
/// use spacey_macros::format_number;
///
/// assert_eq!(format_number!(1234567), "1,234,567");
/// assert_eq!(format_number!(1000), "1,000");
/// assert_eq!(format_number!(42), "42");
/// ```
#[macro_export]
macro_rules! format_number {
    ($num:expr) => {{
        let num = $num as i128;
        let negative = num < 0;
        let abs_num = num.abs();
        let s = abs_num.to_string();
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*c);
        }

        if negative {
            format!("-{}", result)
        } else {
            result
        }
    }};
}

/// Format a percentage.
///
/// # Example
///
/// ```
/// use spacey_macros::format_percent;
///
/// assert_eq!(format_percent!(0.75), "75.0%");
/// assert_eq!(format_percent!(0.333, 2), "33.30%");
/// assert_eq!(format_percent!(1.0), "100.0%");
/// ```
#[macro_export]
macro_rules! format_percent {
    ($value:expr) => {
        format!("{:.1}%", $value as f64 * 100.0)
    };
    ($value:expr, $decimals:expr) => {
        format!("{:.prec$}%", $value as f64 * 100.0, prec = $decimals)
    };
}

/// Join items with a separator.
///
/// # Example
///
/// ```
/// use spacey_macros::join;
///
/// let items = vec!["a", "b", "c"];
/// assert_eq!(join!(items, ", "), "a, b, c");
/// assert_eq!(join!(items, " | "), "a | b | c");
/// ```
#[macro_export]
macro_rules! join {
    ($items:expr, $sep:expr) => {{
        $items
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join($sep)
    }};
}

/// Truncate a string to a maximum length with ellipsis.
///
/// # Example
///
/// ```
/// use spacey_macros::truncate;
///
/// assert_eq!(truncate!("hello world", 8), "hello...");
/// assert_eq!(truncate!("hi", 10), "hi");
/// ```
#[macro_export]
macro_rules! truncate {
    ($s:expr, $max_len:expr) => {{
        let s: &str = $s;
        let max_len: usize = $max_len;
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            s.chars().take(max_len).collect::<String>()
        } else {
            format!("{}...", s.chars().take(max_len - 3).collect::<String>())
        }
    }};
}

/// Pad a string to a minimum length.
///
/// # Example
///
/// ```
/// use spacey_macros::pad_left;
/// use spacey_macros::pad_right;
///
/// assert_eq!(pad_left!("42", 5, '0'), "00042");
/// assert_eq!(pad_right!("hi", 5, ' '), "hi   ");
/// ```
#[macro_export]
macro_rules! pad_left {
    ($s:expr, $width:expr, $char:expr) => {{
        let s: String = $s.to_string();
        let width: usize = $width;
        if s.len() >= width {
            s
        } else {
            let padding: String = ::std::iter::repeat($char).take(width - s.len()).collect();
            format!("{}{}", padding, s)
        }
    }};
}

/// Pad a string on the right to a minimum length.
#[macro_export]
macro_rules! pad_right {
    ($s:expr, $width:expr, $char:expr) => {{
        let s: String = $s.to_string();
        let width: usize = $width;
        if s.len() >= width {
            s
        } else {
            let padding: String = ::std::iter::repeat($char).take(width - s.len()).collect();
            format!("{}{}", s, padding)
        }
    }};
}

/// Convert to hexadecimal string.
///
/// # Example
///
/// ```
/// use spacey_macros::to_hex;
///
/// assert_eq!(to_hex!(255u8), "ff");
/// assert_eq!(to_hex!(4096u32), "1000");
/// ```
#[macro_export]
macro_rules! to_hex {
    ($value:expr) => {
        format!("{:x}", $value)
    };
}

/// Convert to binary string.
///
/// # Example
///
/// ```
/// use spacey_macros::to_binary;
///
/// assert_eq!(to_binary!(5u8), "101");
/// assert_eq!(to_binary!(255u8), "11111111");
/// ```
#[macro_export]
macro_rules! to_binary {
    ($value:expr) => {
        format!("{:b}", $value)
    };
}

/// Create a progress bar string.
///
/// # Example
///
/// ```
/// use spacey_macros::progress_bar;
///
/// assert_eq!(progress_bar!(0.5, 10), "[=====     ]");
/// assert_eq!(progress_bar!(0.0, 10), "[          ]");
/// assert_eq!(progress_bar!(1.0, 10), "[==========]");
/// ```
#[macro_export]
macro_rules! progress_bar {
    ($progress:expr, $width:expr) => {{
        let progress = ($progress as f64).clamp(0.0, 1.0);
        let width = $width;
        let filled = (progress * width as f64).round() as usize;
        let empty = width - filled;
        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    }};
}

/// Indent a multi-line string.
///
/// # Example
///
/// ```
/// use spacey_macros::indent;
///
/// let text = "line1\nline2";
/// assert_eq!(indent!(text, 2), "  line1\n  line2");
/// ```
#[macro_export]
macro_rules! indent {
    ($s:expr, $spaces:expr) => {{
        let s: &str = $s;
        let spaces = " ".repeat($spaces);
        s.lines()
            .map(|line| format!("{}{}", spaces, line))
            .collect::<Vec<_>>()
            .join("\n")
    }};
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes!(512), "512 B");
        assert_eq!(format_bytes!(1024), "1.00 KB");
        assert_eq!(format_bytes!(1536), "1.50 KB");
        assert_eq!(format_bytes!(1048576), "1.00 MB");
        assert_eq!(format_bytes!(1073741824u64), "1.00 GB");
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp!(5, 0, 10), 5);
        assert_eq!(clamp!(-5, 0, 10), 0);
        assert_eq!(clamp!(15, 0, 10), 10);
        assert_eq!(clamp!(0.5, 0.0, 1.0), 0.5);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration!(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration!(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration!(Duration::from_millis(500)), "500ms");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number!(1234567), "1,234,567");
        assert_eq!(format_number!(1000), "1,000");
        assert_eq!(format_number!(42), "42");
        assert_eq!(format_number!(-1234), "-1,234");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent!(0.75), "75.0%");
        assert_eq!(format_percent!(1.0), "100.0%");
    }

    #[test]
    fn test_join() {
        let items = vec!["a", "b", "c"];
        assert_eq!(join!(items, ", "), "a, b, c");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate!("hello world", 8), "hello...");
        assert_eq!(truncate!("hi", 10), "hi");
    }

    #[test]
    fn test_pad() {
        assert_eq!(pad_left!("42", 5, '0'), "00042");
        assert_eq!(pad_right!("hi", 5, ' '), "hi   ");
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(to_hex!(255u8), "ff");
        assert_eq!(to_hex!(16u8), "10");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar!(0.5, 10), "[=====     ]");
        assert_eq!(progress_bar!(0.0, 10), "[          ]");
        assert_eq!(progress_bar!(1.0, 10), "[==========]");
    }
}
