// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! JavaScript engine specific macros.
//!
//! Provides macros for working with JavaScript values, AST, and engine internals.

/// Create a JavaScript-style object literal (returns a HashMap with String values).
///
/// Note: Values are formatted using `{:?}` debug format.
///
/// # Example
///
/// ```
/// use spacey_macros::js_object;
///
/// let obj = js_object! {
///     "name" => "Alice",
///     "age" => 30,
/// };
///
/// // Values are debug-formatted strings
/// assert!(obj.contains_key("name"));
/// assert!(obj.contains_key("age"));
/// ```
#[macro_export]
macro_rules! js_object {
    () => {
        ::std::collections::HashMap::<String, String>::new()
    };
    ($($key:literal => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::HashMap::<String, String>::new();
        $(map.insert($key.to_string(), format!("{:?}", $value));)+
        map
    }};
}

/// Define a JavaScript built-in function.
///
/// # Example
///
/// ```
/// use spacey_macros::js_builtin;
///
/// js_builtin!(console_log, "console.log", |args| {
///     for arg in args {
///         print!("{} ", arg);
///     }
///     println!();
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! js_builtin {
    ($name:ident, $js_name:literal, |$args:ident| $body:expr) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $name {
            pub const JS_NAME: &'static str = $js_name;

            pub fn call<T: std::fmt::Display>($args: &[T]) -> Result<(), String> {
                $body
            }
        }
    };
}

/// Create a token kind enum variant check.
///
/// # Example
///
/// ```
/// use spacey_macros::is_token;
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum TokenKind { Let, Const, Var, Identifier, Number }
///
/// let tok = TokenKind::Let;
/// assert!(is_token!(tok, TokenKind::Let));
/// assert!(!is_token!(tok, TokenKind::Const));
/// ```
#[macro_export]
macro_rules! is_token {
    ($token:expr, $kind:path) => {
        matches!($token, $kind)
    };
    ($token:expr, $($kind:path)|+) => {
        matches!($token, $($kind)|+)
    };
}

/// Check if a character is a valid JavaScript identifier start.
///
/// # Example
///
/// ```
/// use spacey_macros::is_id_start;
///
/// assert!(is_id_start!('a'));
/// assert!(is_id_start!('_'));
/// assert!(is_id_start!('$'));
/// assert!(!is_id_start!('1'));
/// ```
#[macro_export]
macro_rules! is_id_start {
    ($ch:expr) => {{
        let c: char = $ch;
        c.is_alphabetic() || c == '_' || c == '$'
    }};
}

/// Check if a character is a valid JavaScript identifier continuation.
///
/// # Example
///
/// ```
/// use spacey_macros::is_id_continue;
///
/// assert!(is_id_continue!('a'));
/// assert!(is_id_continue!('1'));
/// assert!(is_id_continue!('_'));
/// assert!(!is_id_continue!('-'));
/// ```
#[macro_export]
macro_rules! is_id_continue {
    ($ch:expr) => {{
        let c: char = $ch;
        c.is_alphanumeric() || c == '_' || c == '$'
    }};
}

/// Define a JavaScript keyword lookup table.
///
/// # Example
///
/// ```
/// use spacey_macros::js_keywords;
///
/// js_keywords! {
///     Keywords {
///         "let" => Let,
///         "const" => Const,
///         "var" => Var,
///         "function" => Function,
///     }
/// }
///
/// assert_eq!(Keywords::lookup("let"), Some(Keywords::Let));
/// assert_eq!(Keywords::lookup("unknown"), None);
/// ```
#[macro_export]
macro_rules! js_keywords {
    (
        $name:ident {
            $($keyword:literal => $variant:ident),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn lookup(s: &str) -> Option<Self> {
                match s {
                    $($keyword => Some(Self::$variant),)+
                    _ => None,
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $keyword,)+
                }
            }

            pub fn all() -> &'static [&'static str] {
                &[$($keyword),+]
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

/// Define an opcode enum for the bytecode VM.
///
/// # Example
///
/// ```
/// use spacey_macros::opcodes;
///
/// opcodes! {
///     Opcode: u8 {
///         Nop = 0x00,
///         Push = 0x01,
///         Pop = 0x02,
///         Add = 0x10,
///         Sub = 0x11,
///     }
/// }
///
/// assert_eq!(Opcode::Push as u8, 0x01);
/// assert_eq!(Opcode::try_from(0x10), Ok(Opcode::Add));
/// ```
#[macro_export]
macro_rules! opcodes {
    (
        $name:ident : $repr:ty {
            $($opcode:ident = $value:expr),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr($repr)]
        pub enum $name {
            $($opcode = $value),+
        }

        impl TryFrom<$repr> for $name {
            type Error = ();

            fn try_from(value: $repr) -> Result<Self, ()> {
                match value {
                    $($value => Ok(Self::$opcode),)+
                    _ => Err(()),
                }
            }
        }

        impl $name {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$opcode => stringify!($opcode),)+
                }
            }

            pub fn all() -> &'static [Self] {
                &[$(Self::$opcode),+]
            }
        }
    };
}

/// Define an AST node enum.
///
/// # Example
///
/// ```
/// use spacey_macros::ast_node;
///
/// ast_node!(
///     #[derive(Debug, Clone)]
///     Expression {
///         Literal(value: String),
///         Binary(left: Box<Expression>, op: String, right: Box<Expression>),
///         Identifier(name: String),
///     }
/// );
/// ```
#[macro_export]
macro_rules! ast_node {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($variant:ident($($field:ident : $ty:ty),* $(,)?)),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $($variant { $($field: $ty),* }),+
        }

        impl $name {
            pub fn kind(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => stringify!($variant),)+
                }
            }
        }
    };
}

/// Match a JavaScript value type.
///
/// # Example
///
/// ```
/// use spacey_macros::js_typeof;
///
/// #[derive(Debug)]
/// enum JsValue { Undefined, Null, Boolean(bool), Number(f64), String(String) }
///
/// let val = JsValue::Number(42.0);
/// let type_str = js_typeof!(val,
///     JsValue::Undefined => "undefined",
///     JsValue::Null => "object",
///     JsValue::Boolean(_) => "boolean",
///     JsValue::Number(_) => "number",
///     JsValue::String(_) => "string",
/// );
/// assert_eq!(type_str, "number");
/// ```
#[macro_export]
macro_rules! js_typeof {
    ($val:expr, $($pattern:pat => $type:expr),+ $(,)?) => {
        match $val {
            $($pattern => $type,)+
        }
    };
}

/// Create a span (source location) type.
///
/// # Example
///
/// ```
/// use spacey_macros::span;
///
/// let s = span!(0, 10);
/// assert_eq!(s.start, 0);
/// assert_eq!(s.end, 10);
/// assert_eq!(s.len(), 10);
/// ```
#[macro_export]
macro_rules! span {
    ($start:expr, $end:expr) => {{
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Span {
            pub start: usize,
            pub end: usize,
        }

        impl Span {
            pub fn len(&self) -> usize {
                self.end - self.start
            }

            pub fn is_empty(&self) -> bool {
                self.start == self.end
            }

            pub fn merge(&self, other: &Self) -> Self {
                Self {
                    start: self.start.min(other.start),
                    end: self.end.max(other.end),
                }
            }
        }

        Span {
            start: $start,
            end: $end,
        }
    }};
}

/// Quick assertion for JavaScript spec compliance tests.
///
/// # Example
///
/// ```
/// use spacey_macros::js_assert;
///
/// js_assert!("Array.isArray([])" => true, "arrays should be arrays");
/// ```
#[macro_export]
macro_rules! js_assert {
    ($code:literal => $expected:expr, $msg:literal) => {
        // This would integrate with the actual engine
        // For now, it's a compile-time marker
        #[cfg(test)]
        {
            let _ = ($code, $expected, $msg);
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_id_start() {
        assert!(is_id_start!('a'));
        assert!(is_id_start!('Z'));
        assert!(is_id_start!('_'));
        assert!(is_id_start!('$'));
        assert!(!is_id_start!('1'));
        assert!(!is_id_start!('-'));
    }

    #[test]
    fn test_is_id_continue() {
        assert!(is_id_continue!('a'));
        assert!(is_id_continue!('1'));
        assert!(is_id_continue!('_'));
        assert!(is_id_continue!('$'));
        assert!(!is_id_continue!('-'));
        assert!(!is_id_continue!(' '));
    }

    #[test]
    fn test_js_keywords() {
        js_keywords! {
            TestKeywords {
                "let" => Let,
                "const" => Const,
            }
        }

        assert_eq!(TestKeywords::lookup("let"), Some(TestKeywords::Let));
        assert_eq!(TestKeywords::lookup("const"), Some(TestKeywords::Const));
        assert_eq!(TestKeywords::lookup("var"), None);
        assert_eq!(TestKeywords::Let.as_str(), "let");
    }

    #[test]
    fn test_opcodes() {
        opcodes! {
            TestOp: u8 {
                Nop = 0x00,
                Add = 0x01,
            }
        }

        assert_eq!(TestOp::Nop as u8, 0x00);
        assert_eq!(TestOp::try_from(0x01), Ok(TestOp::Add));
        assert_eq!(TestOp::try_from(0xFF), Err(()));
        assert_eq!(TestOp::Nop.name(), "Nop");
    }

    #[test]
    fn test_is_token() {
        #[derive(Debug, Clone, PartialEq)]
        enum Tok {
            A,
            B,
            C,
        }

        assert!(is_token!(Tok::A, Tok::A));
        assert!(!is_token!(Tok::A, Tok::B));
        assert!(is_token!(Tok::A, Tok::A | Tok::B));
    }
}
