// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `url` module implementation

use crate::error::{NodeError, Result};
use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the url module exports
pub fn create_module() -> Value {
    let exports = HashMap::new();
    Value::NativeObject(exports)
}

/// Parsed URL object
#[derive(Debug, Clone, Default)]
pub struct Url {
    /// Full URL string
    pub href: String,
    /// Protocol (e.g., "https:")
    pub protocol: Option<String>,
    /// Slashes after protocol
    pub slashes: bool,
    /// Auth (username:password)
    pub auth: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// Host (hostname:port)
    pub host: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
    /// Port
    pub port: Option<String>,
    /// Path (pathname + search)
    pub path: Option<String>,
    /// Pathname
    pub pathname: Option<String>,
    /// Search (query string with ?)
    pub search: Option<String>,
    /// Query (search without ?)
    pub query: Option<String>,
    /// Hash (fragment with #)
    pub hash: Option<String>,
}

impl Url {
    /// Create a new URL from components
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();

        obj.insert("href".to_string(), Value::String(self.href.clone()));
        obj.insert(
            "protocol".to_string(),
            self.protocol
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        obj.insert("slashes".to_string(), Value::Boolean(self.slashes));
        obj.insert(
            "auth".to_string(),
            self.auth.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert(
            "host".to_string(),
            self.host.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert(
            "hostname".to_string(),
            self.hostname
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        obj.insert(
            "port".to_string(),
            self.port.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert(
            "pathname".to_string(),
            self.pathname
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        obj.insert(
            "search".to_string(),
            self.search
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        obj.insert(
            "path".to_string(),
            self.path.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert(
            "query".to_string(),
            self.query.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert(
            "hash".to_string(),
            self.hash.clone().map(Value::String).unwrap_or(Value::Null),
        );

        Value::NativeObject(obj)
    }
}

/// Parse a URL string (legacy API)
pub fn parse(url_string: &str, parse_query_string: bool, slashes_denote_host: bool) -> Result<Url> {
    let parsed = url::Url::parse(url_string)
        .map_err(|e| NodeError::Generic(format!("Invalid URL: {}", e)))?;

    let mut url = Url::new();

    url.href = parsed.to_string();
    url.protocol = Some(format!("{}:", parsed.scheme()));
    url.slashes = parsed.scheme() == "http" || parsed.scheme() == "https" || slashes_denote_host;

    if let Some(password) = parsed.password() {
        url.auth = Some(format!("{}:{}", parsed.username(), password));
        url.username = Some(parsed.username().to_string());
        url.password = Some(password.to_string());
    } else if !parsed.username().is_empty() {
        url.auth = Some(parsed.username().to_string());
        url.username = Some(parsed.username().to_string());
    }

    url.hostname = parsed.host_str().map(|s| s.to_string());
    url.port = parsed.port().map(|p| p.to_string());

    if let Some(hostname) = &url.hostname {
        url.host = Some(if let Some(port) = &url.port {
            format!("{}:{}", hostname, port)
        } else {
            hostname.clone()
        });
    }

    url.pathname = Some(parsed.path().to_string());

    if let Some(query) = parsed.query() {
        url.search = Some(format!("?{}", query));
        url.query = if parse_query_string {
            Some(query.to_string())
        } else {
            Some(query.to_string())
        };
    }

    url.path = url.pathname.clone().map(|pathname| {
        if let Some(search) = &url.search {
            format!("{}{}", pathname, search)
        } else {
            pathname
        }
    });

    if let Some(fragment) = parsed.fragment() {
        url.hash = Some(format!("#{}", fragment));
    }

    Ok(url)
}

/// Format a URL object to string (legacy API)
pub fn format(url_obj: &Url) -> String {
    let mut result = String::new();

    if let Some(protocol) = &url_obj.protocol {
        result.push_str(protocol);
        if url_obj.slashes {
            result.push_str("//");
        }
    }

    if let Some(auth) = &url_obj.auth {
        result.push_str(auth);
        result.push('@');
    }

    if let Some(host) = &url_obj.host {
        result.push_str(host);
    } else {
        if let Some(hostname) = &url_obj.hostname {
            result.push_str(hostname);
        }
        if let Some(port) = &url_obj.port {
            result.push(':');
            result.push_str(port);
        }
    }

    if let Some(pathname) = &url_obj.pathname {
        result.push_str(pathname);
    }

    if let Some(search) = &url_obj.search {
        result.push_str(search);
    }

    if let Some(hash) = &url_obj.hash {
        result.push_str(hash);
    }

    result
}

/// Resolve a relative URL against a base URL
pub fn resolve(from: &str, to: &str) -> Result<String> {
    let base = url::Url::parse(from)
        .map_err(|e| NodeError::Generic(format!("Invalid base URL: {}", e)))?;

    let resolved = base
        .join(to)
        .map_err(|e| NodeError::Generic(format!("Failed to resolve URL: {}", e)))?;

    Ok(resolved.to_string())
}

/// Get URL origin
pub fn url_to_http_options(url_string: &str) -> Result<HashMap<String, Value>> {
    let parsed = parse(url_string, false, true)?;
    let mut options = HashMap::new();

    if let Some(protocol) = parsed.protocol {
        options.insert("protocol".to_string(), Value::String(protocol));
    }
    if let Some(hostname) = parsed.hostname {
        options.insert("hostname".to_string(), Value::String(hostname));
    }
    if let Some(port) = parsed.port {
        options.insert("port".to_string(), Value::String(port));
    }
    if let Some(pathname) = parsed.pathname {
        let path = if let Some(search) = parsed.search {
            format!("{}{}", pathname, search)
        } else {
            pathname
        };
        options.insert("path".to_string(), Value::String(path));
    }
    if let Some(auth) = parsed.auth {
        options.insert("auth".to_string(), Value::String(auth));
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let url = parse(
            "https://user:pass@example.com:8080/path?query=1#hash",
            true,
            true,
        )
        .unwrap();

        assert_eq!(url.protocol, Some("https:".to_string()));
        assert_eq!(url.hostname, Some("example.com".to_string()));
        assert_eq!(url.port, Some("8080".to_string()));
        assert_eq!(url.pathname, Some("/path".to_string()));
        assert_eq!(url.search, Some("?query=1".to_string()));
        assert_eq!(url.hash, Some("#hash".to_string()));
        assert_eq!(url.auth, Some("user:pass".to_string()));
    }

    #[test]
    fn test_format() {
        let mut url = Url::new();
        url.protocol = Some("https:".to_string());
        url.slashes = true;
        url.hostname = Some("example.com".to_string());
        url.pathname = Some("/path".to_string());

        let formatted = format(&url);
        assert_eq!(formatted, "https://example.com/path");
    }

    #[test]
    fn test_resolve() {
        let resolved = resolve("https://example.com/path/", "../other").unwrap();
        assert_eq!(resolved, "https://example.com/other");
    }
}
