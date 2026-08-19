// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `zlib` module implementation

use crate::error::{NodeError, Result};
use flate2::Compression;
use flate2::read::{
    DeflateDecoder, DeflateEncoder, GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder,
};
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::io::Read;

/// Create the zlib module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Compression level constants
    exports.insert("Z_NO_COMPRESSION".to_string(), Value::Number(0.0));
    exports.insert("Z_BEST_SPEED".to_string(), Value::Number(1.0));
    exports.insert("Z_BEST_COMPRESSION".to_string(), Value::Number(9.0));
    exports.insert("Z_DEFAULT_COMPRESSION".to_string(), Value::Number(-1.0));

    // Strategy constants
    exports.insert("Z_FILTERED".to_string(), Value::Number(1.0));
    exports.insert("Z_HUFFMAN_ONLY".to_string(), Value::Number(2.0));
    exports.insert("Z_RLE".to_string(), Value::Number(3.0));
    exports.insert("Z_FIXED".to_string(), Value::Number(4.0));
    exports.insert("Z_DEFAULT_STRATEGY".to_string(), Value::Number(0.0));

    Value::NativeObject(exports)
}

/// Compress data using gzip
pub fn gzip_sync(data: &[u8], options: Option<ZlibOptions>) -> Result<Vec<u8>> {
    let level = options.map(|o| o.level).unwrap_or(6);
    let mut encoder = GzEncoder::new(data, Compression::new(level));
    let mut result = Vec::new();
    encoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("Gzip error: {}", e)))?;
    Ok(result)
}

/// Decompress gzip data
pub fn gunzip_sync(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("Gunzip error: {}", e)))?;
    Ok(result)
}

/// Compress data using deflate
pub fn deflate_sync(data: &[u8], options: Option<ZlibOptions>) -> Result<Vec<u8>> {
    let level = options.map(|o| o.level).unwrap_or(6);
    let mut encoder = ZlibEncoder::new(data, Compression::new(level));
    let mut result = Vec::new();
    encoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("Deflate error: {}", e)))?;
    Ok(result)
}

/// Decompress deflate data
pub fn inflate_sync(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("Inflate error: {}", e)))?;
    Ok(result)
}

/// Compress data using raw deflate (no zlib header)
pub fn deflate_raw_sync(data: &[u8], options: Option<ZlibOptions>) -> Result<Vec<u8>> {
    let level = options.map(|o| o.level).unwrap_or(6);
    let mut encoder = DeflateEncoder::new(data, Compression::new(level));
    let mut result = Vec::new();
    encoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("DeflateRaw error: {}", e)))?;
    Ok(result)
}

/// Decompress raw deflate data
pub fn inflate_raw_sync(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|e| NodeError::Generic(format!("InflateRaw error: {}", e)))?;
    Ok(result)
}

/// Zlib options
#[derive(Debug, Clone, Default)]
pub struct ZlibOptions {
    /// Compression level (0-9)
    pub level: u32,
    /// Memory level (1-9)
    pub mem_level: u32,
    /// Strategy
    pub strategy: u32,
    /// Dictionary
    pub dictionary: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_gunzip() {
        let data = b"Hello, World!";
        let compressed = gzip_sync(data, None).unwrap();
        let decompressed = gunzip_sync(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_deflate_inflate() {
        let data = b"Hello, World!";
        let compressed = deflate_sync(data, None).unwrap();
        let decompressed = inflate_sync(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_deflate_raw_inflate_raw() {
        let data = b"Hello, World!";
        let compressed = deflate_raw_sync(data, None).unwrap();
        let decompressed = inflate_raw_sync(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
