// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `string_decoder` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the string_decoder module exports
pub fn create_module() -> Value {
    let exports = HashMap::new();
    Value::NativeObject(exports)
}

/// String decoder for handling multi-byte characters
pub struct StringDecoder {
    encoding: String,
    buffer: Vec<u8>,
}

impl StringDecoder {
    /// Create a new string decoder
    pub fn new(encoding: &str) -> Self {
        Self {
            encoding: encoding.to_lowercase(),
            buffer: Vec::new(),
        }
    }

    /// Write bytes and return decoded string
    pub fn write(&mut self, buf: &[u8]) -> String {
        match self.encoding.as_str() {
            "utf8" | "utf-8" => self.decode_utf8(buf),
            "ascii" => self.decode_ascii(buf),
            "latin1" | "binary" => self.decode_latin1(buf),
            "base64" => self.decode_base64(buf),
            "hex" => self.decode_hex(buf),
            "utf16le" | "ucs2" => self.decode_utf16le(buf),
            _ => self.decode_utf8(buf),
        }
    }

    /// Finish decoding and return remaining string
    pub fn end(&mut self, buf: Option<&[u8]>) -> String {
        let mut result = String::new();

        if let Some(b) = buf {
            result.push_str(&self.write(b));
        }

        // Flush remaining buffer
        if !self.buffer.is_empty() {
            result.push_str(&String::from_utf8_lossy(&self.buffer));
            self.buffer.clear();
        }

        result
    }

    fn decode_utf8(&mut self, buf: &[u8]) -> String {
        // Combine with existing buffer
        self.buffer.extend_from_slice(buf);

        // Try to decode as much as possible
        let mut result = String::new();
        let mut start = 0;

        while start < self.buffer.len() {
            match std::str::from_utf8(&self.buffer[start..]) {
                Ok(s) => {
                    result.push_str(s);
                    self.buffer.clear();
                    return result;
                }
                Err(e) => {
                    let valid_up_to = e.valid_up_to();
                    if valid_up_to > 0 {
                        result.push_str(
                            std::str::from_utf8(&self.buffer[start..start + valid_up_to]).unwrap(),
                        );
                        start += valid_up_to;
                    }

                    // Check if we have an incomplete sequence at the end
                    if e.error_len().is_none() {
                        // Incomplete sequence - keep in buffer
                        self.buffer = self.buffer[start..].to_vec();
                        return result;
                    } else {
                        // Invalid sequence - skip one byte
                        result.push('\u{FFFD}');
                        start += 1;
                    }
                }
            }
        }

        self.buffer.clear();
        result
    }

    fn decode_ascii(&mut self, buf: &[u8]) -> String {
        buf.iter().map(|&b| (b & 0x7f) as char).collect()
    }

    fn decode_latin1(&mut self, buf: &[u8]) -> String {
        buf.iter().map(|&b| b as char).collect()
    }

    fn decode_base64(&mut self, buf: &[u8]) -> String {
        self.buffer.extend_from_slice(buf);

        // Base64 needs 4 bytes at a time
        let usable = (self.buffer.len() / 4) * 4;
        if usable == 0 {
            return String::new();
        }

        let to_decode: Vec<u8> = self.buffer.drain(..usable).collect();
        base64::Engine::decode(&base64::prelude::BASE64_STANDARD, &to_decode)
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .unwrap_or_default()
    }

    fn decode_hex(&mut self, buf: &[u8]) -> String {
        self.buffer.extend_from_slice(buf);

        // Hex needs 2 bytes at a time
        let usable = (self.buffer.len() / 2) * 2;
        if usable == 0 {
            return String::new();
        }

        let to_decode: Vec<u8> = self.buffer.drain(..usable).collect();
        let hex_str = String::from_utf8_lossy(&to_decode);
        hex::decode(hex_str.as_ref())
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .unwrap_or_default()
    }

    fn decode_utf16le(&mut self, buf: &[u8]) -> String {
        self.buffer.extend_from_slice(buf);

        // UTF-16LE needs 2 bytes at a time
        let usable = (self.buffer.len() / 2) * 2;
        if usable == 0 {
            return String::new();
        }

        let to_decode: Vec<u8> = self.buffer.drain(..usable).collect();
        let u16s: Vec<u16> = to_decode
            .chunks(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk.get(1).copied().unwrap_or(0)]))
            .collect();

        String::from_utf16_lossy(&u16s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_decoder() {
        let mut decoder = StringDecoder::new("utf8");
        assert_eq!(decoder.write(b"hello"), "hello");
        assert_eq!(decoder.end(None), "");
    }

    #[test]
    fn test_utf8_split_multibyte() {
        let mut decoder = StringDecoder::new("utf8");
        // UTF-8 for "é" is [0xC3, 0xA9]
        assert_eq!(decoder.write(&[0xC3]), "");
        assert_eq!(decoder.write(&[0xA9]), "é");
    }

    #[test]
    fn test_ascii_decoder() {
        let mut decoder = StringDecoder::new("ascii");
        assert_eq!(decoder.write(b"hello"), "hello");
        // High bit should be stripped
        assert_eq!(decoder.write(&[0xFF]), "\x7F");
    }

    #[test]
    fn test_latin1_decoder() {
        let mut decoder = StringDecoder::new("latin1");
        assert_eq!(decoder.write(&[0xE9]), "é");
    }
}
