//! Date built-in object (ES3 Section 15.9).
//!
//! Provides Date constructor and prototype methods.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

// ============================================================================
// Date Constructor (ES3 Section 15.9.2-3)
// ============================================================================

/// Date() called as a function - returns current date as string.
///
/// ES3 Section 15.9.2
pub fn date_call(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    let now = current_time_ms();
    Ok(Value::String(format_date(now)))
}

/// new Date() constructor - creates a Date object.
///
/// ES3 Section 15.9.3
pub fn date_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time_ms = if args.is_empty() {
        // new Date() - current time
        current_time_ms()
    } else if args.len() == 1 {
        // new Date(value)
        let value = &args[0];
        match value {
            Value::String(s) => parse_date_string(s).unwrap_or(f64::NAN),
            Value::Number(n) => *n,
            _ => value.to_number(),
        }
    } else {
        // new Date(year, month, ...)
        let year = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
        let month = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
        let day = args.get(2).map(|v| v.to_number()).unwrap_or(1.0);
        let hours = args.get(3).map(|v| v.to_number()).unwrap_or(0.0);
        let minutes = args.get(4).map(|v| v.to_number()).unwrap_or(0.0);
        let seconds = args.get(5).map(|v| v.to_number()).unwrap_or(0.0);
        let ms = args.get(6).map(|v| v.to_number()).unwrap_or(0.0);

        make_date(year, month, day, hours, minutes, seconds, ms)
    };

    // In full impl, would create a Date object
    // For now, return the time value as a number
    Ok(Value::Number(time_ms))
}

// ============================================================================
// Date Static Methods (ES3 Section 15.9.4)
// ============================================================================

/// Date.parse(string) - parses a date string.
///
/// ES3 Section 15.9.4.2
pub fn parse(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let s = args.first().map(|v| v.to_js_string()).unwrap_or_default();
    let time = parse_date_string(&s).unwrap_or(f64::NAN);
    Ok(Value::Number(time))
}

/// Date.UTC(year, month, ...) - returns UTC time value.
///
/// ES3 Section 15.9.4.3
pub fn utc(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let year = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    let month = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
    let day = args.get(2).map(|v| v.to_number()).unwrap_or(1.0);
    let hours = args.get(3).map(|v| v.to_number()).unwrap_or(0.0);
    let minutes = args.get(4).map(|v| v.to_number()).unwrap_or(0.0);
    let seconds = args.get(5).map(|v| v.to_number()).unwrap_or(0.0);
    let ms = args.get(6).map(|v| v.to_number()).unwrap_or(0.0);

    let time = make_date(year, month, day, hours, minutes, seconds, ms);
    Ok(Value::Number(time))
}

/// Date.now() - returns current time in milliseconds (ES5, but commonly needed).
pub fn now(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    Ok(Value::Number(current_time_ms()))
}

// ============================================================================
// Date.prototype Methods (ES3 Section 15.9.5)
// ============================================================================

/// Helper to extract time value from args (this value).
fn get_time_value(args: &[Value]) -> f64 {
    args.first().map(|v| v.to_number()).unwrap_or(f64::NAN)
}

/// Date.prototype.toString()
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    Ok(Value::String(format_date(time)))
}

/// Date.prototype.toDateString()
pub fn to_date_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    Ok(Value::String(format_date_only(time)))
}

/// Date.prototype.toTimeString()
pub fn to_time_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    Ok(Value::String(format_time_only(time)))
}

/// Date.prototype.toISOString() (ES5)
pub fn to_iso_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Err("RangeError: Invalid time value".to_string());
    }
    Ok(Value::String(format_iso(time)))
}

/// Date.prototype.toJSON()
pub fn to_json(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    to_iso_string(_frame, args)
}

/// Date.prototype.valueOf() / getTime()
pub fn value_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    Ok(Value::Number(time))
}

/// Date.prototype.getTime()
pub fn get_time(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    value_of(_frame, args)
}

/// Date.prototype.getFullYear()
pub fn get_full_year(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (year, _, _) = ms_to_date_components(time);
    Ok(Value::Number(year as f64))
}

/// Date.prototype.getMonth()
pub fn get_month(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_, month, _) = ms_to_date_components(time);
    Ok(Value::Number(month as f64))
}

/// Date.prototype.getDate()
pub fn get_date(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_, _, day) = ms_to_date_components(time);
    Ok(Value::Number(day as f64))
}

/// Date.prototype.getDay()
pub fn get_day(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    // Days since Unix epoch mod 7, adjusted for Thursday start
    let days = (time / 86400000.0).floor() as i64;
    let day_of_week = ((days + 4) % 7 + 7) % 7; // Thursday is day 0 of epoch, +4 to get Sunday=0
    Ok(Value::Number(day_of_week as f64))
}

/// Date.prototype.getHours()
pub fn get_hours(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (hours, _, _, _) = ms_to_time_components(time);
    Ok(Value::Number(hours as f64))
}

/// Date.prototype.getMinutes()
pub fn get_minutes(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_, minutes, _, _) = ms_to_time_components(time);
    Ok(Value::Number(minutes as f64))
}

/// Date.prototype.getSeconds()
pub fn get_seconds(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_, _, seconds, _) = ms_to_time_components(time);
    Ok(Value::Number(seconds as f64))
}

/// Date.prototype.getMilliseconds()
pub fn get_milliseconds(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    if time.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_, _, _, ms) = ms_to_time_components(time);
    Ok(Value::Number(ms as f64))
}

/// Date.prototype.getTimezoneOffset()
pub fn get_timezone_offset(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    // Return 0 for UTC (simplified)
    Ok(Value::Number(0.0))
}

/// Date.prototype.setTime(time)
pub fn set_time(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(time))
}

/// Date.prototype.setMilliseconds(ms)
pub fn set_milliseconds(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let ms = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || ms.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let (_h, _m, _s, _) = ms_to_time_components(time);
    let new_time = time - (time % 1000.0) + ms;
    Ok(Value::Number(new_time))
}

/// Date.prototype.setSeconds(sec [, ms])
pub fn set_seconds(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let sec = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || sec.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    let ms = args.get(2).map(|v| v.to_number());
    let (_h, _m, _, old_ms) = ms_to_time_components(time);
    let new_ms = ms.unwrap_or(old_ms as f64);
    // Simplified: just adjust seconds in current time
    Ok(Value::Number(
        time - (time % 60000.0) + sec * 1000.0 + new_ms,
    ))
}

/// Date.prototype.setMinutes(min [, sec [, ms]])
pub fn set_minutes(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let min = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || min.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    Ok(Value::Number(time - (time % 3600000.0) + min * 60000.0))
}

/// Date.prototype.setHours(hour [, min [, sec [, ms]]])
pub fn set_hours(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let hour = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || hour.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    Ok(Value::Number(time - (time % 86400000.0) + hour * 3600000.0))
}

/// Date.prototype.setDate(date)
pub fn set_date(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let date = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || date.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    // Simplified implementation
    Ok(Value::Number(time))
}

/// Date.prototype.setMonth(month [, date])
pub fn set_month(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let month = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if time.is_nan() || month.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    // Simplified implementation
    Ok(Value::Number(time))
}

/// Date.prototype.setFullYear(year [, month [, date]])
pub fn set_full_year(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let time = get_time_value(args);
    let year = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    if year.is_nan() {
        return Ok(Value::Number(f64::NAN));
    }
    // Simplified implementation
    Ok(Value::Number(time))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get current time in milliseconds since Unix epoch.
fn current_time_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Parse a date string (simplified).
fn parse_date_string(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try to parse as number (milliseconds)
    if let Ok(n) = s.parse::<f64>() {
        return Some(n);
    }

    // Try ISO 8601 format: YYYY-MM-DDTHH:MM:SS.sssZ
    if let Some(time) = parse_iso_date(s) {
        return Some(time);
    }

    // Try RFC 2822 / JavaScript Date format: "Mon, 25 Dec 1995 13:30:00 GMT"
    if let Some(time) = parse_rfc_date(s) {
        return Some(time);
    }

    // Try simple date format: "MM/DD/YYYY" or "YYYY/MM/DD"
    if let Some(time) = parse_simple_date(s) {
        return Some(time);
    }

    None
}

/// Parse ISO 8601 date format: YYYY-MM-DDTHH:MM:SS.sssZ or YYYY-MM-DD
fn parse_iso_date(s: &str) -> Option<f64> {
    // Check for basic ISO format
    let parts: Vec<&str> = s.split('T').collect();
    let date_part = parts.first()?;
    let time_part = parts.get(1);

    // Parse date: YYYY-MM-DD
    let date_parts: Vec<&str> = date_part.split('-').collect();
    if date_parts.is_empty() {
        return None;
    }

    let year: i32 = date_parts.first()?.parse().ok()?;
    let month: i32 = date_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1) - 1;
    let day: i32 = date_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let (hours, minutes, seconds, ms) = if let Some(time_str) = time_part {
        parse_time_part(time_str)
    } else {
        (0, 0, 0, 0)
    };

    Some(make_date(
        year as f64,
        month as f64,
        day as f64,
        hours as f64,
        minutes as f64,
        seconds as f64,
        ms as f64,
    ))
}

/// Parse time part of ISO date: HH:MM:SS.sssZ
fn parse_time_part(s: &str) -> (i32, i32, i32, i32) {
    // Remove timezone indicator
    let s = s.trim_end_matches('Z').trim_end_matches(|c: char| c == '+' || c == '-' || c.is_ascii_digit());
    let s = s.trim_end_matches(|c: char| c == ':' || c.is_ascii_digit());
    let s_clean: &str = &s.chars().take_while(|c| *c != '+' && *c != '-' && *c != 'Z').collect::<String>();

    let time_parts: Vec<&str> = s_clean.split(':').collect();

    let hours: i32 = time_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minutes: i32 = time_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    // Seconds might have milliseconds: SS.sss
    let (seconds, ms) = if let Some(sec_str) = time_parts.get(2) {
        let sec_parts: Vec<&str> = sec_str.split('.').collect();
        let secs: i32 = sec_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let millis: i32 = sec_parts.get(1).and_then(|s| {
            let ms_str = format!("{:0<3}", s.chars().take(3).collect::<String>());
            ms_str.parse().ok()
        }).unwrap_or(0);
        (secs, millis)
    } else {
        (0, 0)
    };

    (hours, minutes, seconds, ms)
}

/// Parse RFC 2822 style dates
fn parse_rfc_date(s: &str) -> Option<f64> {
    // Very simplified: look for month names
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    // Try to find month and extract date parts
    let mut year: Option<i32> = None;
    let mut month: Option<i32> = None;
    let mut day: Option<i32> = None;

    for part in &parts {
        // Check if it's a month name
        for (i, m) in months.iter().enumerate() {
            if part.eq_ignore_ascii_case(m) {
                month = Some(i as i32);
                break;
            }
        }

        // Check if it's a 4-digit year
        if year.is_none() && part.len() == 4 && part.chars().all(|c| c.is_ascii_digit()) {
            year = part.parse().ok();
        }

        // Check if it's a 1-2 digit day
        if day.is_none() && part.len() <= 2 && part.chars().all(|c| c.is_ascii_digit())
            && let Ok(d) = part.parse::<i32>()
                && (1..=31).contains(&d) {
                    day = Some(d);
                }
    }

    if let (Some(y), Some(m), Some(d)) = (year, month, day) {
        return Some(make_date(
            y as f64,
            m as f64,
            d as f64,
            0.0, 0.0, 0.0, 0.0,
        ));
    }

    None
}

/// Parse simple date formats: MM/DD/YYYY, YYYY/MM/DD
fn parse_simple_date(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }

    let nums: Vec<i32> = parts.iter().filter_map(|p| p.parse().ok()).collect();
    if nums.len() != 3 {
        return None;
    }

    // Determine format based on first number
    let (year, month, day) = if nums[0] > 31 {
        // YYYY/MM/DD
        (nums[0], nums[1] - 1, nums[2])
    } else if nums[2] > 31 {
        // MM/DD/YYYY
        (nums[2], nums[0] - 1, nums[1])
    } else {
        // Assume MM/DD/YYYY
        (nums[2], nums[0] - 1, nums[1])
    };

    Some(make_date(year as f64, month as f64, day as f64, 0.0, 0.0, 0.0, 0.0))
}

/// Make a date from components.
fn make_date(
    year: f64,
    month: f64,
    day: f64,
    hours: f64,
    minutes: f64,
    seconds: f64,
    ms: f64,
) -> f64 {
    if year.is_nan()
        || month.is_nan()
        || day.is_nan()
        || hours.is_nan()
        || minutes.is_nan()
        || seconds.is_nan()
        || ms.is_nan()
    {
        return f64::NAN;
    }

    // Simplified: just compute days from epoch
    // This is not accurate but demonstrates the concept
    let mut y = year as i32;
    if (0..=99).contains(&y) {
        y += 1900;
    }

    // Very rough approximation
    let days_since_epoch = (y - 1970) * 365 + ((month as i32) * 30) + (day as i32) - 1;
    

    (days_since_epoch as f64) * 86400000.0
        + hours * 3600000.0
        + minutes * 60000.0
        + seconds * 1000.0
        + ms
}

/// Convert milliseconds to date components (year, month, day).
fn ms_to_date_components(ms: f64) -> (i32, i32, i32) {
    // Simplified calculation
    let days = (ms / 86400000.0).floor() as i64;
    let year = 1970 + (days / 365) as i32;
    let month = ((days % 365) / 30) as i32;
    let day = ((days % 365) % 30 + 1) as i32;
    (year, month, day)
}

/// Convert milliseconds to time components (hours, minutes, seconds, ms).
fn ms_to_time_components(ms: f64) -> (i32, i32, i32, i32) {
    let total_ms = (ms % 86400000.0) as i64;
    if total_ms < 0 {
        return (0, 0, 0, 0);
    }
    let hours = (total_ms / 3600000) as i32;
    let minutes = ((total_ms % 3600000) / 60000) as i32;
    let seconds = ((total_ms % 60000) / 1000) as i32;
    let millis = (total_ms % 1000) as i32;
    (hours, minutes, seconds, millis)
}

/// Format date as string.
fn format_date(ms: f64) -> String {
    if ms.is_nan() {
        return "Invalid Date".to_string();
    }
    let (year, month, day) = ms_to_date_components(ms);
    let (hours, minutes, seconds, _) = ms_to_time_components(ms);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let day_of_week = (((ms / 86400000.0).floor() as i64 + 4) % 7 + 7) % 7;
    format!(
        "{} {} {:02} {:04} {:02}:{:02}:{:02} GMT+0000",
        days[day_of_week as usize],
        months[month.clamp(0, 11) as usize],
        day,
        year,
        hours,
        minutes,
        seconds
    )
}

/// Format date only.
fn format_date_only(ms: f64) -> String {
    if ms.is_nan() {
        return "Invalid Date".to_string();
    }
    let (year, month, day) = ms_to_date_components(ms);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let day_of_week = (((ms / 86400000.0).floor() as i64 + 4) % 7 + 7) % 7;
    format!(
        "{} {} {:02} {:04}",
        days[day_of_week as usize],
        months[month.clamp(0, 11) as usize],
        day,
        year
    )
}

/// Format time only.
fn format_time_only(ms: f64) -> String {
    if ms.is_nan() {
        return "Invalid Date".to_string();
    }
    let (hours, minutes, seconds, _) = ms_to_time_components(ms);
    format!("{:02}:{:02}:{:02} GMT+0000", hours, minutes, seconds)
}

/// Format as ISO 8601.
fn format_iso(ms: f64) -> String {
    let (year, month, day) = ms_to_date_components(ms);
    let (hours, minutes, seconds, millis) = ms_to_time_components(ms);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year,
        month + 1,
        day,
        hours,
        minutes,
        seconds,
        millis
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::Function;

    fn make_frame() -> CallFrame {
        let func = Function::new(None, vec![], Bytecode::new(), 0);
        CallFrame::new(func, 0)
    }

    #[test]
    fn test_date_now() {
        let mut frame = make_frame();
        let result = now(&mut frame, &[]).unwrap();
        match result {
            Value::Number(n) => assert!(n > 0.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_date_constructor_no_args() {
        let mut frame = make_frame();
        let result = date_constructor(&mut frame, &[]).unwrap();
        match result {
            Value::Number(n) => assert!(n > 0.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_date_constructor_with_ms() {
        let mut frame = make_frame();
        let result = date_constructor(&mut frame, &[Value::Number(0.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_get_time() {
        let mut frame = make_frame();
        let result = get_time(&mut frame, &[Value::Number(1000.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1000.0));
    }

    #[test]
    fn test_get_full_year() {
        let mut frame = make_frame();
        // Test with 0 (Unix epoch = 1970)
        let result = get_full_year(&mut frame, &[Value::Number(0.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1970.0));
    }

    #[test]
    fn test_get_month() {
        let mut frame = make_frame();
        let result = get_month(&mut frame, &[Value::Number(0.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_get_hours() {
        let mut frame = make_frame();
        let result = get_hours(&mut frame, &[Value::Number(3600000.0)]).unwrap(); // 1 hour
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_get_minutes() {
        let mut frame = make_frame();
        let result = get_minutes(&mut frame, &[Value::Number(60000.0)]).unwrap(); // 1 minute
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_get_seconds() {
        let mut frame = make_frame();
        let result = get_seconds(&mut frame, &[Value::Number(1000.0)]).unwrap(); // 1 second
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_get_milliseconds() {
        let mut frame = make_frame();
        let result = get_milliseconds(&mut frame, &[Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_to_string_nan() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(f64::NAN)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "Invalid Date"));
    }

    #[test]
    fn test_to_string_valid() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(0.0)]).unwrap();
        match result {
            Value::String(s) => {
                assert!(s.contains("1970"));
                assert!(s.contains("Jan"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_to_iso_string() {
        let mut frame = make_frame();
        let result = to_iso_string(&mut frame, &[Value::Number(0.0)]).unwrap();
        match result {
            Value::String(s) => {
                assert!(s.contains("1970"));
                assert!(s.contains("T"));
                assert!(s.ends_with("Z"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_to_iso_string_nan() {
        let mut frame = make_frame();
        let result = to_iso_string(&mut frame, &[Value::Number(f64::NAN)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_utc() {
        let mut frame = make_frame();
        let result = utc(
            &mut frame,
            &[
                Value::Number(1970.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        )
        .unwrap();
        match result {
            Value::Number(n) => assert!(n.is_finite()),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_get_timezone_offset() {
        let mut frame = make_frame();
        let result = get_timezone_offset(&mut frame, &[]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_date_call() {
        let mut frame = make_frame();
        let result = date_call(&mut frame, &[]).unwrap();
        match result {
            Value::String(s) => {
                assert!(!s.is_empty());
                assert!(!s.contains("Invalid"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_valid() {
        let mut frame = make_frame();
        // Parse as number string
        let result = parse(&mut frame, &[Value::String("1000".to_string())]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1000.0));
    }

    #[test]
    fn test_parse_invalid() {
        let mut frame = make_frame();
        let result = parse(&mut frame, &[Value::String("not a date".to_string())]).unwrap();
        assert!(matches!(result, Value::Number(n) if n.is_nan()));
    }

    #[test]
    fn test_get_day_epoch() {
        let mut frame = make_frame();
        // Unix epoch was a Thursday (day 4)
        let result = get_day(&mut frame, &[Value::Number(0.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 4.0));
    }

    #[test]
    fn test_set_time() {
        let mut frame = make_frame();
        let result = set_time(&mut frame, &[Value::Number(0.0), Value::Number(5000.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5000.0));
    }
}
