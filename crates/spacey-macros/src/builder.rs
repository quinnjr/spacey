// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builder pattern macros.
//!
//! Provides macros for implementing the builder pattern easily.

/// Generate a builder struct for a type.
///
/// Note: This macro requires the `paste` crate. For a simpler alternative
/// without external dependencies, use `config_struct!` instead.
///
/// # Example
///
/// ```ignore
/// // Requires: paste = "1.0" in Cargo.toml
/// use spacey_macros::builder;
///
/// builder! {
///     Config {
///         name: String,
///         port: u16 = 8080,
///         debug: bool = false,
///     }
/// }
///
/// let config = ConfigBuilder::new()
///     .name("my-app".to_string())
///     .port(3000)
///     .build();
/// ```
#[macro_export]
macro_rules! builder {
    (
        $name:ident {
            $($field:ident : $type:ty $(= $default:expr)?),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type,)+
        }

        paste::paste! {
            #[derive(Debug, Clone, Default)]
            pub struct [<$name Builder>] {
                $($field: Option<$type>,)+
            }

            impl [<$name Builder>] {
                pub fn new() -> Self {
                    Self::default()
                }

                $(
                    pub fn $field(mut self, value: $type) -> Self {
                        self.$field = Some(value);
                        self
                    }
                )+

                pub fn build(self) -> $name {
                    $name {
                        $($field: self.$field $(.or_else(|| Some($default)))?.expect(concat!("missing field: ", stringify!($field))),)+
                    }
                }
            }
        }
    };
}

/// Create a simple builder without the paste crate dependency.
///
/// Note: This macro has limitations with field setters. For most use cases,
/// prefer `config_struct!` which provides a simpler and more reliable API.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::simple_builder;
///
/// simple_builder! {
///     pub struct Person {
///         name: String,
///         age: u32,
///     }
/// }
///
/// let person = Person::builder()
///     .name("Alice")
///     .age(30)
///     .build()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! simple_builder {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $type:ty),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(pub $field: $type,)+
        }
    };
}

/// Create a builder method that sets a field.
///
/// # Example
///
/// ```
/// use spacey_macros::setter;
///
/// pub struct Config {
///     name: String,
///     port: u16,
/// }
///
/// impl Config {
///     pub fn new() -> Self {
///         Self { name: String::new(), port: 8080 }
///     }
///
///     setter!(name, String);
///     setter!(port, u16);
/// }
///
/// let config = Config::new().name("app".to_string()).port(3000);
/// ```
#[macro_export]
macro_rules! setter {
    ($field:ident, $type:ty) => {
        pub fn $field(mut self, value: $type) -> Self {
            self.$field = value;
            self
        }
    };
    ($field:ident, $type:ty, $method_name:ident) => {
        pub fn $method_name(mut self, value: $type) -> Self {
            self.$field = value;
            self
        }
    };
}

/// Create a builder method that appends to a collection.
///
/// # Example
///
/// ```
/// use spacey_macros::appender;
///
/// pub struct Config {
///     items: Vec<String>,
/// }
///
/// impl Config {
///     pub fn new() -> Self {
///         Self { items: Vec::new() }
///     }
///
///     appender!(items, String, add_item);
/// }
///
/// let config = Config::new()
///     .add_item("a".to_string())
///     .add_item("b".to_string());
/// assert_eq!(config.items.len(), 2);
/// ```
#[macro_export]
macro_rules! appender {
    ($field:ident, $item_type:ty, $method_name:ident) => {
        pub fn $method_name(mut self, value: $item_type) -> Self {
            self.$field.push(value);
            self
        }
    };
}

/// Create a builder method that sets an optional field.
///
/// # Example
///
/// ```
/// use spacey_macros::optional_setter;
///
/// pub struct Config {
///     description: Option<String>,
/// }
///
/// impl Config {
///     pub fn new() -> Self {
///         Self { description: None }
///     }
///
///     optional_setter!(description, String);
/// }
///
/// let config = Config::new().description("test".to_string());
/// assert_eq!(config.description, Some("test".to_string()));
/// ```
#[macro_export]
macro_rules! optional_setter {
    ($field:ident, $type:ty) => {
        pub fn $field(mut self, value: $type) -> Self {
            self.$field = Some(value);
            self
        }
    };
}

/// Create a configuration struct with defaults.
///
/// # Example
///
/// ```
/// use spacey_macros::config_struct;
///
/// config_struct! {
///     ServerConfig {
///         host: String = "localhost".to_string(),
///         port: u16 = 8080,
///         workers: usize = 4,
///     }
/// }
///
/// let config = ServerConfig::default();
/// assert_eq!(config.host, "localhost");
/// assert_eq!(config.port, 8080);
/// ```
#[macro_export]
macro_rules! config_struct {
    (
        $name:ident {
            $($field:ident : $type:ty = $default:expr),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type,)+
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field: $default,)+
                }
            }
        }

        impl $name {
            $(
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = value;
                    self
                }
            )+
        }
    };
}

/// Define environment-based configuration.
///
/// # Example
///
/// ```
/// use spacey_macros::env_config;
///
/// env_config! {
///     AppConfig {
///         database_url: String => "DATABASE_URL",
///         port: u16 => "PORT" | 8080,
///         debug: bool => "DEBUG" | false,
///     }
/// }
/// ```
#[macro_export]
macro_rules! env_config {
    (
        $name:ident {
            $($field:ident : $type:ty => $env:literal $(| $default:expr)?),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type,)+
        }

        impl $name {
            pub fn from_env() -> Result<Self, String> {
                Ok(Self {
                    $($field: Self::parse_env::<$type>($env, env_config!(@default $($default)?))?,)+
                })
            }

            fn parse_env<T: ::std::str::FromStr>(
                key: &str,
                default: Option<T>,
            ) -> Result<T, String>
            where
                T::Err: ::std::fmt::Display,
            {
                match ::std::env::var(key) {
                    Ok(val) => val
                        .parse()
                        .map_err(|e| format!("failed to parse {}: {}", key, e)),
                    Err(_) => default.ok_or_else(|| format!("missing env var: {}", key)),
                }
            }
        }
    };
    (@default) => { None };
    (@default $default:expr) => { Some($default) };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_struct() {
        config_struct! {
            TestConfig {
                name: String = "test".to_string(),
                count: u32 = 10,
            }
        }

        let config = TestConfig::default();
        assert_eq!(config.name, "test");
        assert_eq!(config.count, 10);

        let modified = TestConfig::default().name("custom".to_string()).count(20);
        assert_eq!(modified.name, "custom");
        assert_eq!(modified.count, 20);
    }

    #[test]
    fn test_setter() {
        struct Item {
            value: i32,
        }

        impl Item {
            fn new() -> Self {
                Self { value: 0 }
            }

            setter!(value, i32);
        }

        let item = Item::new().value(42);
        assert_eq!(item.value, 42);
    }

    #[test]
    fn test_appender() {
        struct Container {
            items: Vec<i32>,
        }

        impl Container {
            fn new() -> Self {
                Self { items: Vec::new() }
            }

            appender!(items, i32, add);
        }

        let c = Container::new().add(1).add(2).add(3);
        assert_eq!(c.items, vec![1, 2, 3]);
    }
}
