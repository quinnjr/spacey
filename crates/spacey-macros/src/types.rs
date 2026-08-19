// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Type definition macros.
//!
//! Provides macros for defining types with common patterns.

/// Assert at compile time.
///
/// # Example
///
/// ```
/// use spacey_macros::const_assert;
///
/// const_assert!(std::mem::size_of::<u64>() == 8);
/// ```
#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => {
        const _: () = assert!($cond);
    };
    ($cond:expr, $msg:literal) => {
        const _: () = assert!($cond, $msg);
    };
}

/// Create an enum that can convert to/from integers.
///
/// # Example
///
/// ```
/// use spacey_macros::int_enum;
///
/// int_enum! {
///     #[derive(Debug, Clone, Copy, PartialEq)]
///     pub enum Color: u8 {
///         Red = 0,
///         Green = 1,
///         Blue = 2,
///     }
/// }
///
/// assert_eq!(Color::Green as u8, 1);
/// assert_eq!(Color::try_from(2u8).unwrap(), Color::Blue);
/// ```
#[macro_export]
macro_rules! int_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident : $repr:ty {
            $($variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr($repr)]
        $vis enum $name {
            $($variant = $value),+
        }

        impl TryFrom<$repr> for $name {
            type Error = ();

            fn try_from(value: $repr) -> Result<Self, ()> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

/// Create a newtype wrapper with common trait implementations.
///
/// # Example
///
/// ```
/// use spacey_macros::newtype;
///
/// newtype!(UserId, u64);
///
/// let id = UserId(42);
/// assert_eq!(id.0, 42);
/// ```
#[macro_export]
macro_rules! newtype {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub $inner);

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
    ($(#[$meta:meta])* $vis:vis $name:ident, $inner:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(pub $inner);

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

/// Create a string newtype with common trait implementations.
///
/// # Example
///
/// ```
/// use spacey_macros::string_newtype;
///
/// string_newtype!(Email);
///
/// let email = Email::new("test@example.com");
/// assert_eq!(email.as_str(), "test@example.com");
/// ```
#[macro_export]
macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// Create a bitflags type.
///
/// # Example
///
/// ```
/// use spacey_macros::bitflags;
///
/// bitflags! {
///     Permissions: u32 {
///         READ = 0b001,
///         WRITE = 0b010,
///         EXECUTE = 0b100,
///     }
/// }
///
/// let perms = Permissions::READ | Permissions::WRITE;
/// assert!(perms.contains(Permissions::READ));
/// assert!(!perms.contains(Permissions::EXECUTE));
/// ```
#[macro_export]
macro_rules! bitflags {
    (
        $name:ident : $repr:ty {
            $($flag:ident = $value:expr),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name($repr);

        impl $name {
            $(pub const $flag: Self = Self($value);)+

            pub const fn empty() -> Self {
                Self(0)
            }

            pub const fn all() -> Self {
                Self($($value)|+)
            }

            pub const fn bits(&self) -> $repr {
                self.0
            }

            pub const fn contains(&self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            pub fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            pub fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }

            pub fn toggle(&mut self, other: Self) {
                self.0 ^= other.0;
            }

            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }
        }

        impl ::std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl ::std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl ::std::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }
    };
}

/// Define a C-style enum with string conversion.
///
/// # Example
///
/// ```
/// use spacey_macros::str_enum;
///
/// str_enum! {
///     #[derive(Debug, Clone, Copy, PartialEq)]
///     pub enum Status {
///         Active => "active",
///         Inactive => "inactive",
///         Pending => "pending",
///     }
/// }
///
/// assert_eq!(Status::Active.as_str(), "active");
/// assert_eq!(Status::from_str("inactive"), Some(Status::Inactive));
/// ```
#[macro_export]
macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $str:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $str,)+
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($str => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

/// Create a type alias with documentation.
///
/// # Example
///
/// ```
/// use spacey_macros::type_alias;
///
/// type_alias! {
///     /// A unique identifier for a user.
///     pub UserId = u64;
/// }
/// ```
#[macro_export]
macro_rules! type_alias {
    ($(#[$meta:meta])* $vis:vis $name:ident = $type:ty;) => {
        $(#[$meta])*
        $vis type $name = $type;
    };
}

/// Create a tuple struct with a single field.
///
/// # Example
///
/// ```
/// use spacey_macros::tuple_struct;
///
/// tuple_struct!(Point2D, (f64, f64));
///
/// let p = Point2D((1.0, 2.0));
/// ```
#[macro_export]
macro_rules! tuple_struct {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(pub $inner);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_const_assert() {
        const_assert!(1 + 1 == 2);
        const_assert!(std::mem::size_of::<u8>() == 1);
    }

    #[test]
    fn test_int_enum() {
        int_enum! {
            #[derive(Debug, Clone, Copy, PartialEq)]
            enum Status: u8 {
                Success = 0,
                Failed = 1,
                Pending = 2,
            }
        }

        assert_eq!(Status::Success as u8, 0);
        assert_eq!(Status::try_from(1u8).unwrap(), Status::Failed);
        assert!(Status::try_from(99u8).is_err());
    }

    #[test]
    fn test_newtype() {
        newtype!(UserId, u64);

        let id = UserId(42);
        assert_eq!(id.0, 42);
        assert_eq!(*id, 42);

        let id2: UserId = 123u64.into();
        assert_eq!(id2.0, 123);
    }

    #[test]
    fn test_string_newtype() {
        string_newtype!(Email);

        let email = Email::new("test@example.com");
        assert_eq!(email.as_str(), "test@example.com");
        assert_eq!(format!("{}", email), "test@example.com");
    }

    #[test]
    fn test_bitflags() {
        bitflags! {
            Perms: u32 {
                READ = 0b001,
                WRITE = 0b010,
                EXEC = 0b100,
            }
        }

        let p = Perms::READ | Perms::WRITE;
        assert!(p.contains(Perms::READ));
        assert!(p.contains(Perms::WRITE));
        assert!(!p.contains(Perms::EXEC));
    }

    #[test]
    fn test_str_enum() {
        str_enum! {
            #[derive(Debug, Clone, Copy, PartialEq)]
            enum Color {
                Red => "red",
                Green => "green",
                Blue => "blue",
            }
        }

        assert_eq!(Color::Red.as_str(), "red");
        assert_eq!(Color::from_str("green"), Some(Color::Green));
        assert_eq!(Color::from_str("invalid"), None);
    }
}
