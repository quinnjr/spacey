// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `http` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Create the http module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // HTTP methods
    let methods = vec![
        "ACL",
        "BIND",
        "CHECKOUT",
        "CONNECT",
        "COPY",
        "DELETE",
        "GET",
        "HEAD",
        "LINK",
        "LOCK",
        "M-SEARCH",
        "MERGE",
        "MKACTIVITY",
        "MKCALENDAR",
        "MKCOL",
        "MOVE",
        "NOTIFY",
        "OPTIONS",
        "PATCH",
        "POST",
        "PROPFIND",
        "PROPPATCH",
        "PURGE",
        "PUT",
        "REBIND",
        "REPORT",
        "SEARCH",
        "SOURCE",
        "SUBSCRIBE",
        "TRACE",
        "UNBIND",
        "UNLINK",
        "UNLOCK",
        "UNSUBSCRIBE",
    ];
    // METHODS as array-like object
    let mut methods_obj: HashMap<String, Value> = methods
        .iter()
        .enumerate()
        .map(|(i, m)| (i.to_string(), Value::String(m.to_string())))
        .collect();
    methods_obj.insert("length".to_string(), Value::Number(methods.len() as f64));
    exports.insert("METHODS".to_string(), Value::NativeObject(methods_obj));

    // HTTP status codes
    let mut status_codes = HashMap::new();
    status_codes.insert("100".to_string(), Value::String("Continue".to_string()));
    status_codes.insert(
        "101".to_string(),
        Value::String("Switching Protocols".to_string()),
    );
    status_codes.insert("200".to_string(), Value::String("OK".to_string()));
    status_codes.insert("201".to_string(), Value::String("Created".to_string()));
    status_codes.insert("204".to_string(), Value::String("No Content".to_string()));
    status_codes.insert(
        "301".to_string(),
        Value::String("Moved Permanently".to_string()),
    );
    status_codes.insert("302".to_string(), Value::String("Found".to_string()));
    status_codes.insert("304".to_string(), Value::String("Not Modified".to_string()));
    status_codes.insert("400".to_string(), Value::String("Bad Request".to_string()));
    status_codes.insert("401".to_string(), Value::String("Unauthorized".to_string()));
    status_codes.insert("403".to_string(), Value::String("Forbidden".to_string()));
    status_codes.insert("404".to_string(), Value::String("Not Found".to_string()));
    status_codes.insert(
        "405".to_string(),
        Value::String("Method Not Allowed".to_string()),
    );
    status_codes.insert(
        "500".to_string(),
        Value::String("Internal Server Error".to_string()),
    );
    status_codes.insert(
        "501".to_string(),
        Value::String("Not Implemented".to_string()),
    );
    status_codes.insert("502".to_string(), Value::String("Bad Gateway".to_string()));
    status_codes.insert(
        "503".to_string(),
        Value::String("Service Unavailable".to_string()),
    );
    exports.insert(
        "STATUS_CODES".to_string(),
        Value::NativeObject(status_codes),
    );

    // Global agent
    exports.insert("globalAgent".to_string(), Value::Undefined);

    Value::NativeObject(exports)
}

/// HTTP server configuration
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Whether to keep connections alive
    pub keep_alive: bool,
    /// Keep-alive timeout in milliseconds
    pub keep_alive_timeout: u64,
    /// Maximum headers count
    pub max_headers_count: u32,
    /// Request timeout in milliseconds
    pub request_timeout: u64,
    /// Headers timeout in milliseconds
    pub headers_timeout: u64,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            keep_alive: true,
            keep_alive_timeout: 5000,
            max_headers_count: 2000,
            request_timeout: 300000,
            headers_timeout: 60000,
        }
    }
}

/// HTTP request options
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// Request method
    pub method: String,
    /// Hostname
    pub hostname: Option<String>,
    /// Host (hostname:port)
    pub host: Option<String>,
    /// Port
    pub port: Option<u16>,
    /// Path
    pub path: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Request timeout
    pub timeout: Option<u64>,
}

/// Incoming HTTP message (request on server, response on client)
#[derive(Debug)]
pub struct IncomingMessage {
    /// HTTP version
    pub http_version: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Raw headers as flat array
    pub raw_headers: Vec<String>,
    /// HTTP method (for requests)
    pub method: Option<String>,
    /// URL (for requests)
    pub url: Option<String>,
    /// Status code (for responses)
    pub status_code: Option<u16>,
    /// Status message (for responses)
    pub status_message: Option<String>,
    /// Socket info
    pub socket: Option<SocketInfo>,
    /// Whether the message is complete
    pub complete: bool,
}

/// Socket information
#[derive(Debug, Clone)]
pub struct SocketInfo {
    /// Local address
    pub local_address: String,
    /// Local port
    pub local_port: u16,
    /// Remote address
    pub remote_address: String,
    /// Remote port
    pub remote_port: u16,
}

/// Server response
#[derive(Debug)]
pub struct ServerResponse {
    /// Status code
    pub status_code: u16,
    /// Status message
    pub status_message: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Whether headers have been sent
    pub headers_sent: bool,
    /// Whether the response has finished
    pub finished: bool,
}

impl ServerResponse {
    /// Create a new server response
    pub fn new() -> Self {
        Self {
            status_code: 200,
            status_message: "OK".to_string(),
            headers: HashMap::new(),
            headers_sent: false,
            finished: false,
        }
    }

    /// Set status code
    pub fn set_status_code(&mut self, code: u16) {
        self.status_code = code;
        self.status_message = get_status_text(code).to_string();
    }

    /// Set a header
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    /// Get a header
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Remove a header
    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(&name.to_lowercase());
    }

    /// Check if header exists
    pub fn has_header(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    /// Write head (status line and headers)
    pub fn write_head(&mut self, status_code: u16, headers: Option<HashMap<String, String>>) {
        self.status_code = status_code;
        self.status_message = get_status_text(status_code).to_string();

        if let Some(h) = headers {
            for (k, v) in h {
                self.headers.insert(k.to_lowercase(), v);
            }
        }

        self.headers_sent = true;
    }
}

impl Default for ServerResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Get status text for a status code
fn get_status_text(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}

/// HTTP Agent for connection pooling
#[derive(Debug)]
pub struct Agent {
    /// Keep connections alive
    pub keep_alive: bool,
    /// Keep-alive MSECS
    pub keep_alive_msecs: u64,
    /// Max sockets per host
    pub max_sockets: usize,
    /// Max total sockets
    pub max_total_sockets: usize,
    /// Max free sockets
    pub max_free_sockets: usize,
    /// Scheduling strategy
    pub scheduling: String,
    /// Timeout
    pub timeout: Option<u64>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            keep_alive: false,
            keep_alive_msecs: 1000,
            max_sockets: usize::MAX,
            max_total_sockets: usize::MAX,
            max_free_sockets: 256,
            scheduling: "lifo".to_string(),
            timeout: None,
        }
    }
}

/// Create a global agent
pub fn create_global_agent() -> Agent {
    Agent::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_response() {
        let mut res = ServerResponse::new();
        assert_eq!(res.status_code, 200);

        res.set_status_code(404);
        assert_eq!(res.status_code, 404);
        assert_eq!(res.status_message, "Not Found");
    }

    #[test]
    fn test_headers() {
        let mut res = ServerResponse::new();
        res.set_header("Content-Type", "application/json");

        assert!(res.has_header("content-type"));
        assert_eq!(
            res.get_header("content-type"),
            Some(&"application/json".to_string())
        );
    }
}
