//! Number object method implementations.

use crate::runtime::value::Value;

/// Call a number method
pub fn call_number_method(n: f64, method: &str, args: &[Value]) -> Value {
    match method {
        "toString" => {
            let radix = args.first().map(|v| v.to_number() as u32).unwrap_or(10);
            if radix == 10 {
                Value::String(n.to_string())
            } else if (2..=36).contains(&radix) {
                let int_val = n as i64;
                Value::String(format_radix(int_val, radix))
            } else {
                Value::String(n.to_string())
            }
        }
        "toFixed" => {
            let digits = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Value::String(format!("{:.prec$}", n, prec = digits))
        }
        "toExponential" => {
            let digits = args.first().map(|v| v.to_number() as usize);
            match digits {
                Some(d) => Value::String(format!("{:.prec$e}", n, prec = d)),
                None => Value::String(format!("{:e}", n)),
            }
        }
        "toPrecision" => {
            let precision = args.first().map(|v| v.to_number() as usize).unwrap_or(6);
            Value::String(format!("{:.prec$}", n, prec = precision.saturating_sub(1)))
        }
        "valueOf" => Value::Number(n),
        _ => Value::Undefined,
    }
}

/// Format an integer in a given radix
pub fn format_radix(mut n: i64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let negative = n < 0;
    if negative {
        n = -n;
    }
    let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    while n > 0 {
        let digit = (n % radix as i64) as usize;
        result.push(digits.chars().nth(digit).unwrap());
        n /= radix as i64;
    }
    if negative {
        result.push('-');
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_radix_decimal() {
        assert_eq!(format_radix(42, 10), "42");
        assert_eq!(format_radix(-42, 10), "-42");
        assert_eq!(format_radix(0, 10), "0");
    }

    #[test]
    fn test_format_radix_binary() {
        assert_eq!(format_radix(10, 2), "1010");
        assert_eq!(format_radix(255, 2), "11111111");
    }

    #[test]
    fn test_format_radix_hex() {
        assert_eq!(format_radix(255, 16), "ff");
        assert_eq!(format_radix(16, 16), "10");
    }

    #[test]
    fn test_call_number_method_to_string() {
        let result = call_number_method(42.0, "toString", &[]);
        assert!(matches!(result, Value::String(s) if s == "42"));
    }

    #[test]
    fn test_call_number_method_to_fixed() {
        let result = call_number_method(3.14159, "toFixed", &[Value::Number(2.0)]);
        assert!(matches!(result, Value::String(s) if s == "3.14"));
    }

    #[test]
    fn test_call_number_method_value_of() {
        let result = call_number_method(42.0, "valueOf", &[]);
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }
}



