//! Number built-in object (ES3 Section 15.7).
//!
//! Provides Number constructor and prototype methods.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

// ============================================================================
// Number Constants (ES3 Section 15.7.3)
// ============================================================================

/// Number.MAX_VALUE - Largest positive finite value.
pub const MAX_VALUE: f64 = f64::MAX;

/// Number.MIN_VALUE - Smallest positive value (closest to 0).
pub const MIN_VALUE: f64 = f64::MIN_POSITIVE;

/// Number.NaN - Not-a-Number value.
pub const NAN: f64 = f64::NAN;

/// Number.NEGATIVE_INFINITY
pub const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;

/// Number.POSITIVE_INFINITY
pub const POSITIVE_INFINITY: f64 = f64::INFINITY;

// ============================================================================
// Number Constructor (ES3 Section 15.7.1-2)
// ============================================================================

/// Number() constructor - converts value to number.
pub fn number_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
    Ok(Value::Number(n))
}

// ============================================================================
// Number.prototype Methods (ES3 Section 15.7.4)
// ============================================================================

/// Number.prototype.toString(radix) - Returns string representation.
///
/// ES3 Section 15.7.4.2
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = get_this_number(args)?;
    let radix = args.get(1).map(|v| v.to_integer() as i32).unwrap_or(10);

    if !(2..=36).contains(&radix) {
        return Err("RangeError: radix must be between 2 and 36".to_string());
    }

    if radix == 10 {
        return Ok(Value::String(Value::number_to_string(n)));
    }

    // Handle special cases
    if n.is_nan() {
        return Ok(Value::String("NaN".to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::String(if n > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }));
    }
    if n == 0.0 {
        return Ok(Value::String("0".to_string()));
    }

    // Convert integer part
    let is_negative = n < 0.0;
    let abs_n = n.abs();

    if abs_n.fract() == 0.0 && abs_n < i64::MAX as f64 {
        let int_val = abs_n as i64;
        let result = format_radix(int_val, radix as u32);
        let result = if is_negative {
            format!("-{}", result)
        } else {
            result
        };
        return Ok(Value::String(result));
    }

    // For non-integer or very large numbers, fall back to base 10
    Ok(Value::String(Value::number_to_string(n)))
}

/// Number.prototype.toLocaleString() - Returns locale string.
///
/// ES3 Section 15.7.4.3
pub fn to_locale_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    // Simplified: same as toString
    to_string(_frame, args)
}

/// Number.prototype.valueOf() - Returns the number value.
///
/// ES3 Section 15.7.4.4
pub fn value_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = get_this_number(args)?;
    Ok(Value::Number(n))
}

/// Number.prototype.toFixed(digits) - Returns fixed-point notation.
///
/// ES3 Section 15.7.4.5
pub fn to_fixed(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = get_this_number(args)?;
    let digits = args.get(1).map(|v| v.to_integer() as i32).unwrap_or(0);

    if !(0..=20).contains(&digits) {
        return Err("RangeError: digits must be between 0 and 20".to_string());
    }

    if n.is_nan() {
        return Ok(Value::String("NaN".to_string()));
    }
    if n.is_infinite() || n.abs() >= 1e21 {
        return Ok(Value::String(Value::number_to_string(n)));
    }

    Ok(Value::String(format!("{:.1$}", n, digits as usize)))
}

/// Number.prototype.toExponential(digits) - Returns exponential notation.
///
/// ES3 Section 15.7.4.6
pub fn to_exponential(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = get_this_number(args)?;

    if n.is_nan() {
        return Ok(Value::String("NaN".to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::String(if n > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }));
    }

    let digits = args.get(1);

    if digits.is_none() || matches!(digits, Some(Value::Undefined)) {
        // Auto-determine precision
        return Ok(Value::String(format!("{:e}", n)));
    }

    let digits = digits.unwrap().to_integer() as i32;
    if !(0..=20).contains(&digits) {
        return Err("RangeError: digits must be between 0 and 20".to_string());
    }

    Ok(Value::String(format!("{:.1$e}", n, digits as usize)))
}

/// Number.prototype.toPrecision(precision) - Returns string with precision.
///
/// ES3 Section 15.7.4.7
pub fn to_precision(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let n = get_this_number(args)?;

    if n.is_nan() {
        return Ok(Value::String("NaN".to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::String(if n > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }));
    }

    let precision = args.get(1);

    if precision.is_none() || matches!(precision, Some(Value::Undefined)) {
        return Ok(Value::String(Value::number_to_string(n)));
    }

    let precision = precision.unwrap().to_integer() as i32;
    if !(1..=21).contains(&precision) {
        return Err("RangeError: precision must be between 1 and 21".to_string());
    }

    // Use Rust's formatting for precision
    let formatted = format!("{:.1$}", n, (precision - 1) as usize);

    // Trim trailing zeros after decimal point
    let result = if formatted.contains('.') {
        let trimmed = formatted.trim_end_matches('0');
        if trimmed.ends_with('.') {
            trimmed.trim_end_matches('.').to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        formatted
    };

    Ok(Value::String(result))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the number value from 'this' (first argument).
fn get_this_number(args: &[Value]) -> Result<f64, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(*n),
        Some(v) => Ok(v.to_number()),
        None => Ok(0.0),
    }
}

/// Format an integer in the given radix.
fn format_radix(mut n: i64, radix: u32) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    if n == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    let radix = radix as i64;

    while n > 0 {
        let digit = (n % radix) as usize;
        result.push(DIGITS[digit] as char);
        n /= radix;
    }

    result.into_iter().rev().collect()
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
    fn test_number_constants() {
        assert!(MAX_VALUE > 0.0);
        assert!(MIN_VALUE > 0.0);
        assert!(NAN.is_nan());
        assert!(NEGATIVE_INFINITY.is_infinite());
        assert!(POSITIVE_INFINITY.is_infinite());
    }

    #[test]
    fn test_number_constructor() {
        let mut frame = make_frame();
        assert!(matches!(
            number_constructor(&mut frame, &[Value::String("42".to_string())]).unwrap(),
            Value::Number(n) if n == 42.0
        ));
        assert!(matches!(
            number_constructor(&mut frame, &[]).unwrap(),
            Value::Number(n) if n == 0.0
        ));
    }

    #[test]
    fn test_to_string_radix_10() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(255.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "255"));
    }

    #[test]
    fn test_to_string_radix_16() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(255.0), Value::Number(16.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "ff"));
    }

    #[test]
    fn test_to_string_radix_2() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(10.0), Value::Number(2.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "1010"));
    }

    #[test]
    fn test_to_string_invalid_radix() {
        let mut frame = make_frame();
        let result = to_string(&mut frame, &[Value::Number(10.0), Value::Number(1.0)]);
        assert!(result.is_err());

        let result = to_string(&mut frame, &[Value::Number(10.0), Value::Number(37.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_string_special_values() {
        let mut frame = make_frame();
        assert!(matches!(
            to_string(&mut frame, &[Value::Number(f64::NAN)]).unwrap(),
            Value::String(s) if s == "NaN"
        ));
        assert!(matches!(
            to_string(&mut frame, &[Value::Number(f64::INFINITY)]).unwrap(),
            Value::String(s) if s == "Infinity"
        ));
    }

    #[test]
    fn test_value_of() {
        let mut frame = make_frame();
        let result = value_of(&mut frame, &[Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_to_fixed() {
        let mut frame = make_frame();
        let result = to_fixed(&mut frame, &[Value::Number(3.14159), Value::Number(2.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "3.14"));

        let result = to_fixed(&mut frame, &[Value::Number(3.0), Value::Number(2.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s == "3.00"));
    }

    #[test]
    fn test_to_fixed_invalid_digits() {
        let mut frame = make_frame();
        let result = to_fixed(&mut frame, &[Value::Number(3.14), Value::Number(-1.0)]);
        assert!(result.is_err());

        let result = to_fixed(&mut frame, &[Value::Number(3.14), Value::Number(21.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_exponential() {
        let mut frame = make_frame();
        let result =
            to_exponential(&mut frame, &[Value::Number(12345.0), Value::Number(2.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s.contains("e")));
    }

    #[test]
    fn test_to_exponential_special_values() {
        let mut frame = make_frame();
        assert!(matches!(
            to_exponential(&mut frame, &[Value::Number(f64::NAN)]).unwrap(),
            Value::String(s) if s == "NaN"
        ));
        assert!(matches!(
            to_exponential(&mut frame, &[Value::Number(f64::INFINITY)]).unwrap(),
            Value::String(s) if s == "Infinity"
        ));
    }

    #[test]
    fn test_to_precision() {
        let mut frame = make_frame();
        let result =
            to_precision(&mut frame, &[Value::Number(123.456), Value::Number(4.0)]).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_to_precision_invalid() {
        let mut frame = make_frame();
        let result = to_precision(&mut frame, &[Value::Number(123.0), Value::Number(0.0)]);
        assert!(result.is_err());

        let result = to_precision(&mut frame, &[Value::Number(123.0), Value::Number(22.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_radix() {
        assert_eq!(format_radix(255, 16), "ff");
        assert_eq!(format_radix(10, 2), "1010");
        assert_eq!(format_radix(0, 10), "0");
        assert_eq!(format_radix(35, 36), "z");
    }
}
