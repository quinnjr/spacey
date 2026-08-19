// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `stream` module implementation

use crate::modules::events::EventEmitter;
use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the stream module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Stream classes would be registered here
    exports.insert("Readable".to_string(), Value::Undefined);
    exports.insert("Writable".to_string(), Value::Undefined);
    exports.insert("Duplex".to_string(), Value::Undefined);
    exports.insert("Transform".to_string(), Value::Undefined);
    exports.insert("PassThrough".to_string(), Value::Undefined);
    exports.insert("Stream".to_string(), Value::Undefined);

    Value::NativeObject(exports)
}

/// Base stream trait
pub trait Stream {
    /// Destroy the stream
    fn destroy(&mut self);
    /// Check if stream is destroyed
    fn destroyed(&self) -> bool;
}

/// Readable stream state
#[derive(Debug, Default)]
pub struct ReadableState {
    /// High water mark
    pub high_water_mark: usize,
    /// Internal buffer
    pub buffer: Vec<u8>,
    /// Whether stream is flowing
    pub flowing: Option<bool>,
    /// Whether stream has ended
    pub ended: bool,
    /// Whether end has been emitted
    pub end_emitted: bool,
    /// Whether stream is reading
    pub reading: bool,
    /// Whether stream is destroyed
    pub destroyed: bool,
    /// Encoding
    pub encoding: Option<String>,
}

/// Readable stream
#[derive(Debug, Default)]
pub struct Readable {
    /// Event emitter
    pub emitter: EventEmitter,
    /// Stream state
    pub state: ReadableState,
}

impl Readable {
    /// Create a new readable stream
    pub fn new(options: Option<ReadableOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            emitter: EventEmitter::new(),
            state: ReadableState {
                high_water_mark: options.high_water_mark.unwrap_or(16 * 1024),
                flowing: None,
                ..Default::default()
            },
        }
    }

    /// Read data from the stream
    pub fn read(&mut self, size: Option<usize>) -> Option<Vec<u8>> {
        if self.state.destroyed || self.state.ended {
            return None;
        }

        let size = size.unwrap_or(self.state.buffer.len());
        if size == 0 || self.state.buffer.is_empty() {
            return None;
        }

        let to_read = size.min(self.state.buffer.len());
        let data: Vec<u8> = self.state.buffer.drain(..to_read).collect();
        Some(data)
    }

    /// Push data to the stream
    pub fn push(&mut self, chunk: Option<Vec<u8>>) -> bool {
        match chunk {
            Some(data) => {
                self.state.buffer.extend(data);
                self.state.buffer.len() < self.state.high_water_mark
            }
            None => {
                self.state.ended = true;
                false
            }
        }
    }

    /// Unshift data back to the front
    pub fn unshift(&mut self, chunk: Vec<u8>) {
        let mut new_buffer = chunk;
        new_buffer.extend(std::mem::take(&mut self.state.buffer));
        self.state.buffer = new_buffer;
    }

    /// Set encoding
    pub fn set_encoding(&mut self, encoding: &str) -> &mut Self {
        self.state.encoding = Some(encoding.to_string());
        self
    }

    /// Pause the stream
    pub fn pause(&mut self) -> &mut Self {
        self.state.flowing = Some(false);
        self
    }

    /// Resume the stream
    pub fn resume(&mut self) -> &mut Self {
        self.state.flowing = Some(true);
        self
    }

    /// Check if stream is paused
    pub fn is_paused(&self) -> bool {
        self.state.flowing == Some(false)
    }

    /// Pipe to a writable stream
    pub fn pipe<'a>(&mut self, destination: &'a mut Writable) -> &'a mut Writable {
        // Simplified pipe implementation
        destination
    }

    /// Unpipe from a writable stream
    pub fn unpipe(&mut self, _destination: Option<&Writable>) -> &mut Self {
        self
    }
}

impl Stream for Readable {
    fn destroy(&mut self) {
        self.state.destroyed = true;
    }

    fn destroyed(&self) -> bool {
        self.state.destroyed
    }
}

/// Readable stream options
#[derive(Debug, Default)]
pub struct ReadableOptions {
    /// High water mark
    pub high_water_mark: Option<usize>,
    /// Object mode
    pub object_mode: bool,
    /// Encoding
    pub encoding: Option<String>,
    /// Auto destroy
    pub auto_destroy: bool,
}

/// Writable stream state
#[derive(Debug, Default)]
pub struct WritableState {
    /// High water mark
    pub high_water_mark: usize,
    /// Whether stream is finished
    pub finished: bool,
    /// Whether stream is destroyed
    pub destroyed: bool,
    /// Whether stream is writable
    pub writable: bool,
    /// Pending writes
    pub buffer: Vec<Vec<u8>>,
    /// Whether currently writing
    pub writing: bool,
    /// Whether stream is corked
    pub corked: u32,
}

/// Writable stream
#[derive(Debug, Default)]
pub struct Writable {
    /// Event emitter
    pub emitter: EventEmitter,
    /// Stream state
    pub state: WritableState,
}

impl Writable {
    /// Create a new writable stream
    pub fn new(options: Option<WritableOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            emitter: EventEmitter::new(),
            state: WritableState {
                high_water_mark: options.high_water_mark.unwrap_or(16 * 1024),
                writable: true,
                ..Default::default()
            },
        }
    }

    /// Write data to the stream
    pub fn write(&mut self, chunk: Vec<u8>, _encoding: Option<&str>) -> bool {
        if self.state.destroyed || self.state.finished {
            return false;
        }

        self.state.buffer.push(chunk);
        self.state.buffer.iter().map(|b| b.len()).sum::<usize>() < self.state.high_water_mark
    }

    /// End the stream
    pub fn end(&mut self, chunk: Option<Vec<u8>>) {
        if let Some(data) = chunk {
            self.write(data, None);
        }
        self.state.finished = true;
    }

    /// Cork the stream (buffer writes)
    pub fn cork(&mut self) {
        self.state.corked += 1;
    }

    /// Uncork the stream (flush buffered writes)
    pub fn uncork(&mut self) {
        if self.state.corked > 0 {
            self.state.corked -= 1;
        }
    }

    /// Set default encoding
    pub fn set_default_encoding(&mut self, _encoding: &str) -> &mut Self {
        self
    }
}

impl Stream for Writable {
    fn destroy(&mut self) {
        self.state.destroyed = true;
    }

    fn destroyed(&self) -> bool {
        self.state.destroyed
    }
}

/// Writable stream options
#[derive(Debug, Default)]
pub struct WritableOptions {
    /// High water mark
    pub high_water_mark: Option<usize>,
    /// Object mode
    pub object_mode: bool,
    /// Encoding
    pub encoding: Option<String>,
    /// Auto destroy
    pub auto_destroy: bool,
}

/// Duplex stream (both readable and writable)
#[derive(Debug, Default)]
pub struct Duplex {
    /// Readable part
    pub readable: Readable,
    /// Writable part
    pub writable: Writable,
}

impl Duplex {
    /// Create a new duplex stream
    pub fn new() -> Self {
        Self {
            readable: Readable::new(None),
            writable: Writable::new(None),
        }
    }
}

/// Transform stream (duplex stream that transforms data)
#[derive(Debug, Default)]
pub struct Transform {
    /// Base duplex
    pub duplex: Duplex,
}

impl Transform {
    /// Create a new transform stream
    pub fn new() -> Self {
        Self {
            duplex: Duplex::new(),
        }
    }
}

/// PassThrough stream (transform that passes data unchanged)
#[derive(Debug, Default)]
pub struct PassThrough {
    /// Base transform
    pub transform: Transform,
}

impl PassThrough {
    /// Create a new passthrough stream
    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
        }
    }
}

/// Pipe streams together
pub fn pipeline(_streams: &[&dyn Stream]) -> Result<(), String> {
    // Simplified pipeline implementation
    Ok(())
}

/// Check if value is a stream
pub fn is_stream(_value: &Value) -> bool {
    // Would check if value has stream properties
    false
}

/// Check if value is readable
pub fn is_readable(_value: &Value) -> bool {
    false
}

/// Check if value is writable
pub fn is_writable(_value: &Value) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readable_push_read() {
        let mut readable = Readable::new(None);

        readable.push(Some(b"hello".to_vec()));
        readable.push(Some(b" world".to_vec()));

        let data = readable.read(None).unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_writable_write() {
        let mut writable = Writable::new(None);

        let result = writable.write(b"hello".to_vec(), None);
        assert!(result);

        writable.end(Some(b" world".to_vec()));
        assert!(writable.state.finished);
    }

    #[test]
    fn test_readable_pause_resume() {
        let mut readable = Readable::new(None);

        assert!(!readable.is_paused());
        readable.pause();
        assert!(readable.is_paused());
        readable.resume();
        assert!(!readable.is_paused());
    }
}
