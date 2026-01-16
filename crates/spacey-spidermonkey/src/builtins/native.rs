//! Native function implementations.
//!
//! This module defines builtin function IDs and their implementations.

use crate::Error;
use crate::runtime::value::Value;

/// Builtin function IDs.
///
/// Each native function has a unique ID that the VM uses to dispatch calls.
/// The IDs are grouped by category:
/// - 0-9: Global functions (parseInt, parseFloat, etc.)
/// - 10-49: Math methods
/// - 50-69: Number methods
/// - 70-119: String methods
/// - 120-169: Array methods
/// - 170-199: Object methods
/// - 200-209: Console methods
/// - 210-219: JSON methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[allow(missing_docs)] // Variants are self-documenting by name
pub enum BuiltinId {
    // Global functions
    ParseInt = 0,
    ParseFloat = 1,
    IsNaN = 2,
    IsFinite = 3,
    EncodeURI = 4,
    DecodeURI = 5,
    EncodeURIComponent = 6,
    DecodeURIComponent = 7,

    // Math methods (10-49)
    MathAbs = 10,
    MathCeil = 11,
    MathFloor = 12,
    MathRound = 13,
    MathMax = 14,
    MathMin = 15,
    MathPow = 16,
    MathSqrt = 17,
    MathExp = 18,
    MathLog = 19,
    MathSin = 20,
    MathCos = 21,
    MathTan = 22,
    MathAsin = 23,
    MathAcos = 24,
    MathAtan = 25,
    MathAtan2 = 26,
    MathRandom = 27,
    MathSign = 28,
    MathTrunc = 29,

    // Number methods (50-69)
    NumberToString = 50,
    NumberToFixed = 51,
    NumberToExponential = 52,
    NumberToPrecision = 53,
    NumberValueOf = 54,

    // String methods (70-119)
    StringCharAt = 70,
    StringCharCodeAt = 71,
    StringConcat = 72,
    StringIndexOf = 73,
    StringLastIndexOf = 74,
    StringSlice = 75,
    StringSubstring = 76,
    StringSubstr = 77,
    StringToLowerCase = 78,
    StringToUpperCase = 79,
    StringTrim = 80,
    StringSplit = 81,
    StringReplace = 82,
    StringMatch = 83,
    StringSearch = 84,
    StringRepeat = 85,
    StringStartsWith = 86,
    StringEndsWith = 87,
    StringIncludes = 88,
    StringPadStart = 89,
    StringPadEnd = 90,

    // Array methods (120-169)
    ArrayPush = 120,
    ArrayPop = 121,
    ArrayShift = 122,
    ArrayUnshift = 123,
    ArraySlice = 124,
    ArraySplice = 125,
    ArrayConcat = 126,
    ArrayJoin = 127,
    ArrayReverse = 128,
    ArraySort = 129,
    ArrayIndexOf = 130,
    ArrayLastIndexOf = 131,
    ArrayForEach = 132,
    ArrayMap = 133,
    ArrayFilter = 134,
    ArrayEvery = 135,
    ArraySome = 136,
    ArrayReduce = 137,
    ArrayReduceRight = 138,
    ArrayIsArray = 139,
    ArrayFrom = 140,

    // Object methods (170-199)
    ObjectKeys = 170,
    ObjectValues = 171,
    ObjectEntries = 172,
    ObjectHasOwnProperty = 173,
    ObjectToString = 174,
    ObjectValueOf = 175,
    ObjectCreate = 176,
    ObjectDefineProperty = 177,
    ObjectGetOwnPropertyDescriptor = 178,
    ObjectGetPrototypeOf = 179,

    // Console methods (200-209)
    ConsoleLog = 200,
    ConsoleError = 201,
    ConsoleWarn = 202,

    // JSON methods (210-219)
    JsonParse = 210,
    JsonStringify = 211,
}

impl BuiltinId {
    /// Creates a BuiltinId from a raw u16 value.
    pub fn from_u16(id: u16) -> Option<Self> {
        match id {
            0 => Some(Self::ParseInt),
            1 => Some(Self::ParseFloat),
            2 => Some(Self::IsNaN),
            3 => Some(Self::IsFinite),
            4 => Some(Self::EncodeURI),
            5 => Some(Self::DecodeURI),
            6 => Some(Self::EncodeURIComponent),
            7 => Some(Self::DecodeURIComponent),
            10 => Some(Self::MathAbs),
            11 => Some(Self::MathCeil),
            12 => Some(Self::MathFloor),
            13 => Some(Self::MathRound),
            14 => Some(Self::MathMax),
            15 => Some(Self::MathMin),
            16 => Some(Self::MathPow),
            17 => Some(Self::MathSqrt),
            18 => Some(Self::MathExp),
            19 => Some(Self::MathLog),
            20 => Some(Self::MathSin),
            21 => Some(Self::MathCos),
            22 => Some(Self::MathTan),
            23 => Some(Self::MathAsin),
            24 => Some(Self::MathAcos),
            25 => Some(Self::MathAtan),
            26 => Some(Self::MathAtan2),
            27 => Some(Self::MathRandom),
            28 => Some(Self::MathSign),
            29 => Some(Self::MathTrunc),
            50 => Some(Self::NumberToString),
            51 => Some(Self::NumberToFixed),
            52 => Some(Self::NumberToExponential),
            53 => Some(Self::NumberToPrecision),
            54 => Some(Self::NumberValueOf),
            70 => Some(Self::StringCharAt),
            71 => Some(Self::StringCharCodeAt),
            72 => Some(Self::StringConcat),
            73 => Some(Self::StringIndexOf),
            74 => Some(Self::StringLastIndexOf),
            75 => Some(Self::StringSlice),
            76 => Some(Self::StringSubstring),
            77 => Some(Self::StringSubstr),
            78 => Some(Self::StringToLowerCase),
            79 => Some(Self::StringToUpperCase),
            80 => Some(Self::StringTrim),
            81 => Some(Self::StringSplit),
            82 => Some(Self::StringReplace),
            83 => Some(Self::StringMatch),
            84 => Some(Self::StringSearch),
            85 => Some(Self::StringRepeat),
            86 => Some(Self::StringStartsWith),
            87 => Some(Self::StringEndsWith),
            88 => Some(Self::StringIncludes),
            89 => Some(Self::StringPadStart),
            90 => Some(Self::StringPadEnd),
            120 => Some(Self::ArrayPush),
            121 => Some(Self::ArrayPop),
            122 => Some(Self::ArrayShift),
            123 => Some(Self::ArrayUnshift),
            124 => Some(Self::ArraySlice),
            125 => Some(Self::ArraySplice),
            126 => Some(Self::ArrayConcat),
            127 => Some(Self::ArrayJoin),
            128 => Some(Self::ArrayReverse),
            129 => Some(Self::ArraySort),
            130 => Some(Self::ArrayIndexOf),
            131 => Some(Self::ArrayLastIndexOf),
            132 => Some(Self::ArrayForEach),
            133 => Some(Self::ArrayMap),
            134 => Some(Self::ArrayFilter),
            135 => Some(Self::ArrayEvery),
            136 => Some(Self::ArraySome),
            137 => Some(Self::ArrayReduce),
            138 => Some(Self::ArrayReduceRight),
            139 => Some(Self::ArrayIsArray),
            140 => Some(Self::ArrayFrom),
            170 => Some(Self::ObjectKeys),
            171 => Some(Self::ObjectValues),
            172 => Some(Self::ObjectEntries),
            173 => Some(Self::ObjectHasOwnProperty),
            174 => Some(Self::ObjectToString),
            175 => Some(Self::ObjectValueOf),
            176 => Some(Self::ObjectCreate),
            177 => Some(Self::ObjectDefineProperty),
            178 => Some(Self::ObjectGetOwnPropertyDescriptor),
            179 => Some(Self::ObjectGetPrototypeOf),
            200 => Some(Self::ConsoleLog),
            201 => Some(Self::ConsoleError),
            202 => Some(Self::ConsoleWarn),
            210 => Some(Self::JsonParse),
            211 => Some(Self::JsonStringify),
            _ => None,
        }
    }
}

/// Call a builtin function by ID.
pub fn call_builtin(id: BuiltinId, this: &Value, args: &[Value]) -> Result<Value, Error> {
    match id {
        // Global functions
        BuiltinId::ParseInt => builtin_parse_int(args),
        BuiltinId::ParseFloat => builtin_parse_float(args),
        BuiltinId::IsNaN => builtin_is_nan(args),
        BuiltinId::IsFinite => builtin_is_finite(args),
        BuiltinId::EncodeURI => builtin_encode_uri(args),
        BuiltinId::DecodeURI => builtin_decode_uri(args),
        BuiltinId::EncodeURIComponent => builtin_encode_uri_component(args),
        BuiltinId::DecodeURIComponent => builtin_decode_uri_component(args),

        // Math methods
        BuiltinId::MathAbs => builtin_math_abs(args),
        BuiltinId::MathCeil => builtin_math_ceil(args),
        BuiltinId::MathFloor => builtin_math_floor(args),
        BuiltinId::MathRound => builtin_math_round(args),
        BuiltinId::MathMax => builtin_math_max(args),
        BuiltinId::MathMin => builtin_math_min(args),
        BuiltinId::MathPow => builtin_math_pow(args),
        BuiltinId::MathSqrt => builtin_math_sqrt(args),
        BuiltinId::MathExp => builtin_math_exp(args),
        BuiltinId::MathLog => builtin_math_log(args),
        BuiltinId::MathSin => builtin_math_sin(args),
        BuiltinId::MathCos => builtin_math_cos(args),
        BuiltinId::MathTan => builtin_math_tan(args),
        BuiltinId::MathAsin => builtin_math_asin(args),
        BuiltinId::MathAcos => builtin_math_acos(args),
        BuiltinId::MathAtan => builtin_math_atan(args),
        BuiltinId::MathAtan2 => builtin_math_atan2(args),
        BuiltinId::MathRandom => builtin_math_random(args),
        BuiltinId::MathSign => builtin_math_sign(args),
        BuiltinId::MathTrunc => builtin_math_trunc(args),

        // Number methods
        BuiltinId::NumberToString => builtin_number_to_string(this, args),
        BuiltinId::NumberToFixed => builtin_number_to_fixed(this, args),
        BuiltinId::NumberToExponential => builtin_number_to_exponential(this, args),
        BuiltinId::NumberToPrecision => builtin_number_to_precision(this, args),
        BuiltinId::NumberValueOf => builtin_number_value_of(this),

        // String methods
        BuiltinId::StringCharAt => builtin_string_char_at(this, args),
        BuiltinId::StringCharCodeAt => builtin_string_char_code_at(this, args),
        BuiltinId::StringConcat => builtin_string_concat(this, args),
        BuiltinId::StringIndexOf => builtin_string_index_of(this, args),
        BuiltinId::StringLastIndexOf => builtin_string_last_index_of(this, args),
        BuiltinId::StringSlice => builtin_string_slice(this, args),
        BuiltinId::StringSubstring => builtin_string_substring(this, args),
        BuiltinId::StringSubstr => builtin_string_substr(this, args),
        BuiltinId::StringToLowerCase => builtin_string_to_lower_case(this),
        BuiltinId::StringToUpperCase => builtin_string_to_upper_case(this),
        BuiltinId::StringTrim => builtin_string_trim(this),
        BuiltinId::StringSplit => builtin_string_split(this, args),
        BuiltinId::StringReplace => builtin_string_replace(this, args),
        BuiltinId::StringMatch => builtin_string_match(this, args),
        BuiltinId::StringSearch => builtin_string_search(this, args),
        BuiltinId::StringRepeat => builtin_string_repeat(this, args),
        BuiltinId::StringStartsWith => builtin_string_starts_with(this, args),
        BuiltinId::StringEndsWith => builtin_string_ends_with(this, args),
        BuiltinId::StringIncludes => builtin_string_includes(this, args),
        BuiltinId::StringPadStart => builtin_string_pad_start(this, args),
        BuiltinId::StringPadEnd => builtin_string_pad_end(this, args),

        // Array methods - these need object access which we don't have here
        BuiltinId::ArrayIsArray => builtin_array_is_array(args),
        _ => Ok(Value::Undefined), // Unimplemented builtin
    }
}

// ==================== Global Functions ====================

fn to_number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => *n,
        Value::Boolean(true) => 1.0,
        Value::Boolean(false) => 0.0,
        Value::Null => 0.0,
        Value::Undefined => f64::NAN,
        Value::String(s) => s.trim().parse().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

fn to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) if n.is_nan() => "NaN".to_string(),
        Value::Number(n) if n.is_infinite() => {
            if *n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
        }
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Undefined => "undefined".to_string(),
        _ => "[object Object]".to_string(),
    }
}

fn builtin_parse_int(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    let radix = args.get(1).map(to_number).unwrap_or(10.0) as i32;

    let string = string.trim();
    if string.is_empty() {
        return Ok(Value::Number(f64::NAN));
    }

    let (string, radix) = if radix == 0 || radix == 10 {
        if string.starts_with("0x") || string.starts_with("0X") {
            (&string[2..], 16)
        } else {
            // ES5+ doesn't auto-octal, so all other cases use radix 10
            (string, 10)
        }
    } else if radix == 16 && (string.starts_with("0x") || string.starts_with("0X")) {
        (&string[2..], 16)
    } else {
        (string, radix)
    };

    if !(2..=36).contains(&radix) {
        return Ok(Value::Number(f64::NAN));
    }

    match i64::from_str_radix(string, radix as u32) {
        Ok(n) => Ok(Value::Number(n as f64)),
        Err(_) => {
            // Try parsing as much as possible
            let mut result = 0i64;
            let mut found_digit = false;
            let negative = string.starts_with('-');
            let start = if negative || string.starts_with('+') {
                1
            } else {
                0
            };

            for c in string[start..].chars() {
                let digit = c.to_digit(radix as u32);
                match digit {
                    Some(d) => {
                        found_digit = true;
                        result = result * radix as i64 + d as i64;
                    }
                    None => break,
                }
            }

            if found_digit {
                Ok(Value::Number(if negative { -result } else { result } as f64))
            } else {
                Ok(Value::Number(f64::NAN))
            }
        }
    }
}

fn builtin_parse_float(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    let trimmed = string.trim();

    if trimmed.is_empty() {
        return Ok(Value::Number(f64::NAN));
    }

    match trimmed.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n)),
        Err(_) => {
            // Try parsing as much as possible from the start
            let mut end = 0;
            let mut has_dot = false;
            let mut has_exp = false;

            let chars: Vec<char> = trimmed.chars().collect();
            let mut i = 0;

            // Optional sign
            if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                i += 1;
            }

            // Digits before decimal
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
                end = i;
            }

            // Decimal point
            if i < chars.len() && chars[i] == '.' {
                has_dot = true;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                    end = i;
                }
            }

            // Exponent
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let _exp_start = i;
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                let digit_start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i > digit_start {
                    has_exp = true;
                    end = i;
                }
            }

            let _ = has_dot;
            let _ = has_exp;

            if end == 0 {
                Ok(Value::Number(f64::NAN))
            } else {
                let parsed: String = chars[..end].iter().collect();
                Ok(Value::Number(parsed.parse().unwrap_or(f64::NAN)))
            }
        }
    }
}

fn builtin_is_nan(args: &[Value]) -> Result<Value, Error> {
    let num = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Boolean(num.is_nan()))
}

fn builtin_is_finite(args: &[Value]) -> Result<Value, Error> {
    let num = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Boolean(num.is_finite()))
}

fn builtin_encode_uri(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    // Simplified: just return the string for now
    // Full implementation would encode special characters
    Ok(Value::String(percent_encode(&string, false)))
}

fn builtin_decode_uri(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    Ok(Value::String(percent_decode(&string)))
}

fn builtin_encode_uri_component(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    Ok(Value::String(percent_encode(&string, true)))
}

fn builtin_decode_uri_component(args: &[Value]) -> Result<Value, Error> {
    let string = args.first().map(to_string).unwrap_or_default();
    Ok(Value::String(percent_decode(&string)))
}

fn percent_encode(s: &str, component: bool) -> String {
    let mut result = String::new();
    for c in s.chars() {
        let is_unreserved = c.is_ascii_alphanumeric() || "-_.!~*'()".contains(c);
        let is_reserved = !component && ";,/?:@&=+$#".contains(c);
        if is_unreserved || is_reserved {
            result.push(c);
        } else {
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn percent_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&String::from_utf8_lossy(&bytes[i + 1..i + 3]), 16)
        {
            result.push(byte);
            i += 3;
            continue;
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}

// ==================== Math Functions ====================

fn builtin_math_abs(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.abs()))
}

fn builtin_math_ceil(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.ceil()))
}

fn builtin_math_floor(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.floor()))
}

fn builtin_math_round(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.round()))
}

fn builtin_math_max(args: &[Value]) -> Result<Value, Error> {
    if args.is_empty() {
        return Ok(Value::Number(f64::NEG_INFINITY));
    }
    let mut max = f64::NEG_INFINITY;
    for arg in args {
        let n = to_number(arg);
        if n.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if n > max {
            max = n;
        }
    }
    Ok(Value::Number(max))
}

fn builtin_math_min(args: &[Value]) -> Result<Value, Error> {
    if args.is_empty() {
        return Ok(Value::Number(f64::INFINITY));
    }
    let mut min = f64::INFINITY;
    for arg in args {
        let n = to_number(arg);
        if n.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if n < min {
            min = n;
        }
    }
    Ok(Value::Number(min))
}

fn builtin_math_pow(args: &[Value]) -> Result<Value, Error> {
    let base = args.first().map(to_number).unwrap_or(f64::NAN);
    let exp = args.get(1).map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(base.powf(exp)))
}

fn builtin_math_sqrt(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.sqrt()))
}

fn builtin_math_exp(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.exp()))
}

fn builtin_math_log(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.ln()))
}

fn builtin_math_sin(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.sin()))
}

fn builtin_math_cos(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.cos()))
}

fn builtin_math_tan(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.tan()))
}

fn builtin_math_asin(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.asin()))
}

fn builtin_math_acos(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.acos()))
}

fn builtin_math_atan(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.atan()))
}

fn builtin_math_atan2(args: &[Value]) -> Result<Value, Error> {
    let y = args.first().map(to_number).unwrap_or(f64::NAN);
    let x = args.get(1).map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(y.atan2(x)))
}

fn builtin_math_random(_args: &[Value]) -> Result<Value, Error> {
    // Simple random using system time - not cryptographically secure
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Simple LCG
    let random = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % (1 << 31)) as f64
        / (1u64 << 31) as f64;
    Ok(Value::Number(random))
}

fn builtin_math_sign(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    if n.is_nan() {
        Ok(Value::Number(f64::NAN))
    } else if n > 0.0 {
        Ok(Value::Number(1.0))
    } else if n < 0.0 {
        Ok(Value::Number(-1.0))
    } else {
        Ok(Value::Number(n)) // Preserves +0 and -0
    }
}

fn builtin_math_trunc(args: &[Value]) -> Result<Value, Error> {
    let n = args.first().map(to_number).unwrap_or(f64::NAN);
    Ok(Value::Number(n.trunc()))
}

// ==================== Number Methods ====================

fn builtin_number_to_string(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let n = to_number(this);
    let radix = args.first().map(to_number).unwrap_or(10.0) as u32;

    if !(2..=36).contains(&radix) {
        return Err(Error::RangeError("radix must be between 2 and 36".into()));
    }

    if radix == 10 {
        return Ok(Value::String(to_string(&Value::Number(n))));
    }

    if n.is_nan() {
        return Ok(Value::String("NaN".into()));
    }
    if n.is_infinite() {
        return Ok(Value::String(
            if n > 0.0 { "Infinity" } else { "-Infinity" }.into(),
        ));
    }

    // Convert integer part to different radix
    let negative = n < 0.0;
    let mut int_part = n.abs().trunc() as i64;
    let mut result = String::new();

    if int_part == 0 {
        result.push('0');
    } else {
        while int_part > 0 {
            let digit = (int_part % radix as i64) as u32;
            result.push(char::from_digit(digit, radix).unwrap());
            int_part /= radix as i64;
        }
        result = result.chars().rev().collect();
    }

    if negative {
        result.insert(0, '-');
    }

    Ok(Value::String(result))
}

fn builtin_number_to_fixed(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let n = to_number(this);
    let digits = args.first().map(to_number).unwrap_or(0.0) as usize;

    if digits > 100 {
        return Err(Error::RangeError(
            "toFixed() digits argument must be between 0 and 100".into(),
        ));
    }

    Ok(Value::String(format!("{:.1$}", n, digits)))
}

fn builtin_number_to_exponential(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let n = to_number(this);
    let digits = args.first().map(to_number);

    match digits {
        Some(d) if !(0.0..=100.0).contains(&d) => Err(Error::RangeError(
            "toExponential() digits must be between 0 and 100".into(),
        )),
        Some(d) => Ok(Value::String(format!("{:.1$e}", n, d as usize))),
        None => Ok(Value::String(format!("{:e}", n))),
    }
}

fn builtin_number_to_precision(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let n = to_number(this);
    let precision = args.first().map(to_number);

    match precision {
        Some(p) if !(1.0..=100.0).contains(&p) => Err(Error::RangeError(
            "toPrecision() precision must be between 1 and 100".into(),
        )),
        Some(p) => Ok(Value::String(format!("{:.1$}", n, p as usize - 1))),
        None => Ok(Value::String(to_string(&Value::Number(n)))),
    }
}

fn builtin_number_value_of(this: &Value) -> Result<Value, Error> {
    Ok(Value::Number(to_number(this)))
}

// ==================== String Methods ====================

fn builtin_string_char_at(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let index = args.first().map(to_number).unwrap_or(0.0) as usize;

    Ok(Value::String(
        s.chars()
            .nth(index)
            .map(|c| c.to_string())
            .unwrap_or_default(),
    ))
}

fn builtin_string_char_code_at(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let index = args.first().map(to_number).unwrap_or(0.0) as usize;

    Ok(Value::Number(
        s.chars()
            .nth(index)
            .map(|c| c as u32 as f64)
            .unwrap_or(f64::NAN),
    ))
}

fn builtin_string_concat(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let mut result = to_string(this);
    for arg in args {
        result.push_str(&to_string(arg));
    }
    Ok(Value::String(result))
}

fn builtin_string_index_of(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let start = args.get(1).map(to_number).unwrap_or(0.0) as usize;

    if start >= s.len() {
        if search.is_empty() && start == s.len() {
            return Ok(Value::Number(s.len() as f64));
        }
        return Ok(Value::Number(-1.0));
    }

    Ok(Value::Number(
        s[start..]
            .find(&search)
            .map(|i| (i + start) as f64)
            .unwrap_or(-1.0),
    ))
}

fn builtin_string_last_index_of(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let end = args.get(1).map(to_number).unwrap_or(f64::INFINITY);

    let end = if end.is_nan() || end.is_infinite() {
        s.len()
    } else {
        (end as usize).min(s.len())
    };

    Ok(Value::Number(
        s[..end].rfind(&search).map(|i| i as f64).unwrap_or(-1.0),
    ))
}

fn builtin_string_slice(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let len = s.len() as i64;

    let start = args.first().map(to_number).unwrap_or(0.0) as i64;
    let end = args.get(1).map(|v| to_number(v) as i64).unwrap_or(len);

    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;
    let end = if end < 0 {
        (len + end).max(0)
    } else {
        end.min(len)
    } as usize;

    if start >= end {
        return Ok(Value::String(String::new()));
    }

    Ok(Value::String(
        s.chars().skip(start).take(end - start).collect(),
    ))
}

fn builtin_string_substring(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let len = s.len();

    let start = args.first().map(to_number).unwrap_or(0.0);
    let end = args.get(1).map(to_number).unwrap_or(len as f64);

    let start = if start.is_nan() {
        0
    } else {
        (start as usize).min(len)
    };
    let end = if end.is_nan() {
        0
    } else {
        (end as usize).min(len)
    };

    let (start, end) = if start > end {
        (end, start)
    } else {
        (start, end)
    };

    Ok(Value::String(
        s.chars().skip(start).take(end - start).collect(),
    ))
}

fn builtin_string_substr(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let len = s.len() as i64;

    let start = args.first().map(to_number).unwrap_or(0.0) as i64;
    let length = args.get(1).map(|v| to_number(v) as i64).unwrap_or(len);

    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;
    let length = length.max(0) as usize;

    Ok(Value::String(s.chars().skip(start).take(length).collect()))
}

fn builtin_string_to_lower_case(this: &Value) -> Result<Value, Error> {
    Ok(Value::String(to_string(this).to_lowercase()))
}

fn builtin_string_to_upper_case(this: &Value) -> Result<Value, Error> {
    Ok(Value::String(to_string(this).to_uppercase()))
}

fn builtin_string_trim(this: &Value) -> Result<Value, Error> {
    Ok(Value::String(to_string(this).trim().to_string()))
}

fn builtin_string_split(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let separator = args.first();
    let limit = args
        .get(1)
        .map(|v| to_number(v) as usize)
        .unwrap_or(usize::MAX);

    // Without separator, return array with single element
    if separator.is_none() || matches!(separator, Some(Value::Undefined)) {
        // Would need to create array object here
        // For now, just return the string
        return Ok(Value::String(s));
    }

    let sep = to_string(separator.unwrap());
    if sep.is_empty() {
        // Split into characters
        let chars: Vec<String> = s.chars().take(limit).map(|c| c.to_string()).collect();
        // Would create array here
        return Ok(Value::String(chars.join(",")));
    }

    let parts: Vec<&str> = s.split(&sep).take(limit).collect();
    // Would create array here
    Ok(Value::String(parts.join(",")))
}

fn builtin_string_replace(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let replacement = args.get(1).map(to_string).unwrap_or_default();

    // Simple implementation: replace first occurrence only
    Ok(Value::String(s.replacen(&search, &replacement, 1)))
}

fn builtin_string_match(this: &Value, _args: &[Value]) -> Result<Value, Error> {
    // Would need regex support
    Ok(Value::String(to_string(this)))
}

fn builtin_string_search(this: &Value, args: &[Value]) -> Result<Value, Error> {
    // Simplified: just indexOf
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    Ok(Value::Number(
        s.find(&search).map(|i| i as f64).unwrap_or(-1.0),
    ))
}

fn builtin_string_repeat(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let count = args.first().map(to_number).unwrap_or(0.0) as usize;

    if count == 0 {
        return Ok(Value::String(String::new()));
    }

    Ok(Value::String(s.repeat(count)))
}

fn builtin_string_starts_with(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let start = args.get(1).map(to_number).unwrap_or(0.0) as usize;

    if start >= s.len() {
        return Ok(Value::Boolean(search.is_empty()));
    }

    Ok(Value::Boolean(s[start..].starts_with(&search)))
}

fn builtin_string_ends_with(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let end = args
        .get(1)
        .map(|v| to_number(v) as usize)
        .unwrap_or(s.len());

    let end = end.min(s.len());
    Ok(Value::Boolean(s[..end].ends_with(&search)))
}

fn builtin_string_includes(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let search = args.first().map(to_string).unwrap_or_default();
    let start = args.get(1).map(to_number).unwrap_or(0.0) as usize;

    if start >= s.len() {
        return Ok(Value::Boolean(search.is_empty()));
    }

    Ok(Value::Boolean(s[start..].contains(&search)))
}

fn builtin_string_pad_start(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let target_len = args.first().map(to_number).unwrap_or(0.0) as usize;
    let pad_string = args
        .get(1)
        .map(to_string)
        .unwrap_or_else(|| " ".to_string());

    if target_len <= s.len() || pad_string.is_empty() {
        return Ok(Value::String(s));
    }

    let pad_len = target_len - s.len();
    let mut padding = String::new();
    while padding.len() < pad_len {
        padding.push_str(&pad_string);
    }
    padding.truncate(pad_len);

    Ok(Value::String(format!("{}{}", padding, s)))
}

fn builtin_string_pad_end(this: &Value, args: &[Value]) -> Result<Value, Error> {
    let s = to_string(this);
    let target_len = args.first().map(to_number).unwrap_or(0.0) as usize;
    let pad_string = args
        .get(1)
        .map(to_string)
        .unwrap_or_else(|| " ".to_string());

    if target_len <= s.len() || pad_string.is_empty() {
        return Ok(Value::String(s));
    }

    let pad_len = target_len - s.len();
    let mut padding = String::new();
    while padding.len() < pad_len {
        padding.push_str(&pad_string);
    }
    padding.truncate(pad_len);

    Ok(Value::String(format!("{}{}", s, padding)))
}

// ==================== Array Methods ====================

fn builtin_array_is_array(args: &[Value]) -> Result<Value, Error> {
    // Simplified check - would need proper array detection
    match args.first() {
        Some(Value::Object(_)) => Ok(Value::Boolean(true)), // Simplified
        _ => Ok(Value::Boolean(false)),
    }
}
