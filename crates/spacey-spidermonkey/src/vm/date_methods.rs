//! Date object method implementations.

use crate::runtime::value::Value;

/// Call a Date method
pub fn call_date_method(timestamp: f64, method: &str, _args: &[Value]) -> Value {
    if timestamp.is_nan() || timestamp.is_infinite() {
        return Value::Number(f64::NAN);
    }

    // Helper to extract date components from timestamp
    // Timestamp is milliseconds since Unix epoch (Jan 1, 1970)
    let ms = timestamp as i64;
    let secs = ms / 1000;
    let millis = (ms % 1000) as f64;

    // Days since epoch
    let days_since_epoch = secs / 86400;

    // Calculate year, month, day
    let (year, month, day, day_of_week) = days_to_ymd(days_since_epoch);

    // Calculate hours, minutes, seconds
    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    match method {
        "getTime" | "valueOf" => Value::Number(timestamp),
        "getFullYear" => Value::Number(year as f64),
        "getMonth" => Value::Number(month as f64), // 0-indexed
        "getDate" => Value::Number(day as f64),
        "getDay" => Value::Number(day_of_week as f64),
        "getHours" => Value::Number(hours as f64),
        "getMinutes" => Value::Number(minutes as f64),
        "getSeconds" => Value::Number(seconds as f64),
        "getMilliseconds" => Value::Number(millis),
        "toString" => Value::String(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year,
            month + 1,
            day,
            hours,
            minutes,
            seconds,
            millis as i64
        )),
        "toDateString" => Value::String(format!("{:04}-{:02}-{:02}", year, month + 1, day)),
        "toTimeString" => Value::String(format!("{:02}:{:02}:{:02}", hours, minutes, seconds)),
        _ => Value::Undefined,
    }
}

/// Convert days since Unix epoch to (year, month, day, day_of_week)
pub fn days_to_ymd(days: i64) -> (i32, i32, i32, i32) {
    // Simplified date calculation
    // Note: This is a basic implementation, may have edge cases
    let remaining_days = days + 719468; // Days from year 0 to 1970

    // Calculate year
    let era = if remaining_days >= 0 {
        remaining_days
    } else {
        remaining_days - 146096
    } / 146097;
    let doe = (remaining_days - era * 146097) as i32; // Day of era
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // Year of era
    let year = yoe + (era as i32) * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // Day of year
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    // Day of week (0 = Sunday)
    let day_of_week = ((days + 4) % 7 + 7) % 7;

    (year, month - 1, day, day_of_week as i32) // month is 0-indexed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_to_ymd_epoch() {
        // Day 0 = Jan 1, 1970 (Thursday = 4)
        let (year, month, day, dow) = days_to_ymd(0);
        assert_eq!(year, 1970);
        assert_eq!(month, 0); // January
        assert_eq!(day, 1);
        assert_eq!(dow, 4); // Thursday
    }

    #[test]
    fn test_call_date_method_get_time() {
        let result = call_date_method(1000.0, "getTime", &[]);
        assert!(matches!(result, Value::Number(n) if n == 1000.0));
    }

    #[test]
    fn test_call_date_method_nan() {
        let result = call_date_method(f64::NAN, "getTime", &[]);
        assert!(matches!(result, Value::Number(n) if n.is_nan()));
    }
}
