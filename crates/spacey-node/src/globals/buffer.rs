// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `Buffer` class implementation
//!
//! Buffer is used for handling binary data in Node.js.

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Rust representation of a Node.js Buffer
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The underlying byte data
    data: Vec<u8>,
}

impl Buffer {
    /// Create a new empty buffer with the given size
    pub fn alloc(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// Create a new buffer with the given size, filled with a value
    pub fn alloc_unsafe(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
        }
    }

    /// Create a buffer from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
        }
    }

    /// Create a buffer from a string with the given encoding
    pub fn from_string(s: &str, encoding: &str) -> Self {
        let data = match encoding {
            "utf8" | "utf-8" => s.as_bytes().to_vec(),
            "ascii" => s.bytes().map(|b| b & 0x7f).collect(),
            "base64" => {
                base64::Engine::decode(&base64::prelude::BASE64_STANDARD, s).unwrap_or_default()
            }
            "hex" => hex::decode(s).unwrap_or_default(),
            "latin1" | "binary" => s.bytes().collect(),
            _ => s.as_bytes().to_vec(),
        };
        Self { data }
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to string with the given encoding
    pub fn to_string(&self, encoding: &str) -> String {
        match encoding {
            "utf8" | "utf-8" => String::from_utf8_lossy(&self.data).to_string(),
            "ascii" => self.data.iter().map(|&b| (b & 0x7f) as char).collect(),
            "base64" => base64::Engine::encode(&base64::prelude::BASE64_STANDARD, &self.data),
            "hex" => hex::encode(&self.data),
            "latin1" | "binary" => self.data.iter().map(|&b| b as char).collect(),
            _ => String::from_utf8_lossy(&self.data).to_string(),
        }
    }

    /// Get a byte at the given index
    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Set a byte at the given index
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        if index < self.data.len() {
            self.data[index] = value;
            true
        } else {
            false
        }
    }

    /// Get a slice of the buffer
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let start = start.min(self.data.len());
        let end = end.min(self.data.len());
        Self {
            data: self.data[start..end].to_vec(),
        }
    }

    /// Copy data from another buffer
    pub fn copy_from(
        &mut self,
        source: &Buffer,
        target_start: usize,
        source_start: usize,
        source_end: usize,
    ) -> usize {
        let source_start = source_start.min(source.data.len());
        let source_end = source_end.min(source.data.len());
        let target_start = target_start.min(self.data.len());

        let bytes_to_copy = (source_end - source_start).min(self.data.len() - target_start);

        self.data[target_start..target_start + bytes_to_copy]
            .copy_from_slice(&source.data[source_start..source_start + bytes_to_copy]);

        bytes_to_copy
    }

    /// Fill the buffer with a value
    pub fn fill(&mut self, value: u8, start: usize, end: usize) {
        let start = start.min(self.data.len());
        let end = end.min(self.data.len());
        self.data[start..end].fill(value);
    }

    /// Compare two buffers
    pub fn compare(&self, other: &Buffer) -> i32 {
        match self.data.cmp(&other.data) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    /// Check if the buffer equals another
    pub fn equals(&self, other: &Buffer) -> bool {
        self.data == other.data
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to the underlying bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consume and return the underlying bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Write a string to the buffer
    pub fn write(&mut self, s: &str, offset: usize, length: usize, encoding: &str) -> usize {
        let bytes = match encoding {
            "utf8" | "utf-8" => s.as_bytes().to_vec(),
            "ascii" => s.bytes().map(|b| b & 0x7f).collect(),
            "base64" => {
                base64::Engine::decode(&base64::prelude::BASE64_STANDARD, s).unwrap_or_default()
            }
            "hex" => hex::decode(s).unwrap_or_default(),
            _ => s.as_bytes().to_vec(),
        };

        let offset = offset.min(self.data.len());
        let length = length.min(bytes.len()).min(self.data.len() - offset);

        self.data[offset..offset + length].copy_from_slice(&bytes[..length]);
        length
    }

    /// Read a signed 8-bit integer
    pub fn read_int8(&self, offset: usize) -> Option<i8> {
        self.data.get(offset).map(|&b| b as i8)
    }

    /// Read an unsigned 8-bit integer
    pub fn read_uint8(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    /// Read a big-endian 16-bit unsigned integer
    pub fn read_uint16_be(&self, offset: usize) -> Option<u16> {
        if offset + 2 <= self.data.len() {
            Some(u16::from_be_bytes([
                self.data[offset],
                self.data[offset + 1],
            ]))
        } else {
            None
        }
    }

    /// Read a little-endian 16-bit unsigned integer
    pub fn read_uint16_le(&self, offset: usize) -> Option<u16> {
        if offset + 2 <= self.data.len() {
            Some(u16::from_le_bytes([
                self.data[offset],
                self.data[offset + 1],
            ]))
        } else {
            None
        }
    }

    /// Read a big-endian 32-bit unsigned integer
    pub fn read_uint32_be(&self, offset: usize) -> Option<u32> {
        if offset + 4 <= self.data.len() {
            Some(u32::from_be_bytes([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ]))
        } else {
            None
        }
    }

    /// Read a little-endian 32-bit unsigned integer
    pub fn read_uint32_le(&self, offset: usize) -> Option<u32> {
        if offset + 4 <= self.data.len() {
            Some(u32::from_le_bytes([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ]))
        } else {
            None
        }
    }

    /// Read a big-endian 64-bit float
    pub fn read_double_be(&self, offset: usize) -> Option<f64> {
        if offset + 8 <= self.data.len() {
            let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
            Some(f64::from_be_bytes(bytes))
        } else {
            None
        }
    }

    /// Read a little-endian 64-bit float
    pub fn read_double_le(&self, offset: usize) -> Option<f64> {
        if offset + 8 <= self.data.len() {
            let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
            Some(f64::from_le_bytes(bytes))
        } else {
            None
        }
    }
}

/// Create the Buffer class for the JavaScript runtime
pub fn create_buffer_class() -> Value {
    let mut buffer_class = HashMap::new();

    // Buffer.alloc(size) - static method
    // Buffer.allocUnsafe(size) - static method
    // Buffer.from(data) - static method
    // Buffer.isBuffer(obj) - static method
    // Buffer.concat(list) - static method
    // Buffer.byteLength(string, encoding) - static method

    // These would be native functions
    buffer_class.insert("poolSize".to_string(), Value::Number(8192.0));

    Value::NativeObject(buffer_class)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_alloc() {
        let buf = Buffer::alloc(10);
        assert_eq!(buf.len(), 10);
        assert!(buf.as_bytes().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_buffer_from_string() {
        let buf = Buffer::from_string("hello", "utf8");
        assert_eq!(buf.to_string("utf8"), "hello");
    }

    #[test]
    fn test_buffer_slice() {
        let buf = Buffer::from_string("hello world", "utf8");
        let slice = buf.slice(0, 5);
        assert_eq!(slice.to_string("utf8"), "hello");
    }

    #[test]
    fn test_buffer_hex() {
        let buf = Buffer::from_bytes(&[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(buf.to_string("hex"), "deadbeef");
    }
}
