// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `crypto` module implementation

use crate::error::{NodeError, Result};
use ring::digest::{self, Algorithm, Context as DigestContext};
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use spacey_spidermonkey::Value;
use std::collections::HashMap;

/// Create the crypto module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // List of supported hash algorithms
    let hashes = vec![
        "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-224", "sha3-256", "sha3-384",
        "sha3-512",
    ];

    // getHashes as array-like object
    let mut hashes_obj: HashMap<String, Value> = hashes
        .iter()
        .enumerate()
        .map(|(i, h)| (i.to_string(), Value::String(h.to_string())))
        .collect();
    hashes_obj.insert("length".to_string(), Value::Number(hashes.len() as f64));
    exports.insert("getHashes".to_string(), Value::NativeObject(hashes_obj));

    // List of supported ciphers
    let ciphers = vec![
        "aes-128-cbc",
        "aes-192-cbc",
        "aes-256-cbc",
        "aes-128-gcm",
        "aes-256-gcm",
        "chacha20-poly1305",
    ];

    // getCiphers as array-like object
    let mut ciphers_obj: HashMap<String, Value> = ciphers
        .iter()
        .enumerate()
        .map(|(i, c)| (i.to_string(), Value::String(c.to_string())))
        .collect();
    ciphers_obj.insert("length".to_string(), Value::Number(ciphers.len() as f64));
    exports.insert("getCiphers".to_string(), Value::NativeObject(ciphers_obj));

    Value::NativeObject(exports)
}

/// Get the ring algorithm for a hash name
fn get_algorithm(name: &str) -> Result<&'static Algorithm> {
    match name.to_lowercase().as_str() {
        "sha1" => Ok(&digest::SHA1_FOR_LEGACY_USE_ONLY),
        "sha256" | "sha-256" => Ok(&digest::SHA256),
        "sha384" | "sha-384" => Ok(&digest::SHA384),
        "sha512" | "sha-512" => Ok(&digest::SHA512),
        _ => Err(NodeError::Crypto(format!(
            "Unsupported algorithm: {}",
            name
        ))),
    }
}

/// Hash implementation
pub struct Hash {
    context: DigestContext,
    algorithm_name: String,
}

impl Hash {
    /// Create a new hash
    pub fn new(algorithm: &str) -> Result<Self> {
        let alg = get_algorithm(algorithm)?;
        Ok(Self {
            context: DigestContext::new(alg),
            algorithm_name: algorithm.to_string(),
        })
    }

    /// Update the hash with data
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.context.update(data);
        self
    }

    /// Finalize and get the digest
    pub fn digest(self, encoding: Option<&str>) -> String {
        let result = self.context.finish();
        let bytes = result.as_ref();

        match encoding {
            Some("hex") | None => hex::encode(bytes),
            Some("base64") => base64::Engine::encode(&base64::prelude::BASE64_STANDARD, bytes),
            Some("binary") | Some("latin1") => bytes.iter().map(|&b| b as char).collect(),
            _ => hex::encode(bytes),
        }
    }
}

/// HMAC implementation
pub struct Hmac {
    key: hmac::Key,
    context: Option<hmac::Context>,
}

impl Hmac {
    /// Create a new HMAC
    pub fn new(algorithm: &str, key: &[u8]) -> Result<Self> {
        let alg = match algorithm.to_lowercase().as_str() {
            "sha1" => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            "sha256" | "sha-256" => hmac::HMAC_SHA256,
            "sha384" | "sha-384" => hmac::HMAC_SHA384,
            "sha512" | "sha-512" => hmac::HMAC_SHA512,
            _ => {
                return Err(NodeError::Crypto(format!(
                    "Unsupported HMAC algorithm: {}",
                    algorithm
                )));
            }
        };

        let key = hmac::Key::new(alg, key);
        let context = hmac::Context::with_key(&key);

        Ok(Self {
            key,
            context: Some(context),
        })
    }

    /// Update with data
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        if let Some(ctx) = &mut self.context {
            ctx.update(data);
        }
        self
    }

    /// Finalize and get the digest
    pub fn digest(mut self, encoding: Option<&str>) -> String {
        let tag = self.context.take().unwrap().sign();
        let bytes = tag.as_ref();

        match encoding {
            Some("hex") | None => hex::encode(bytes),
            Some("base64") => base64::Engine::encode(&base64::prelude::BASE64_STANDARD, bytes),
            Some("binary") | Some("latin1") => bytes.iter().map(|&b| b as char).collect(),
            _ => hex::encode(bytes),
        }
    }
}

/// Generate random bytes
pub fn random_bytes(size: usize) -> Result<Vec<u8>> {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; size];
    rng.fill(&mut bytes)
        .map_err(|_| NodeError::Crypto("Failed to generate random bytes".to_string()))?;
    Ok(bytes)
}

/// Generate random bytes synchronously (same as random_bytes for now)
pub fn random_bytes_sync(size: usize) -> Result<Vec<u8>> {
    random_bytes(size)
}

/// Fill a buffer with random bytes
pub fn random_fill(buffer: &mut [u8]) -> Result<()> {
    let rng = SystemRandom::new();
    rng.fill(buffer)
        .map_err(|_| NodeError::Crypto("Failed to fill random bytes".to_string()))?;
    Ok(())
}

/// Generate a random UUID v4
pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate random integer in range [min, max)
pub fn random_int(min: i64, max: i64) -> Result<i64> {
    if min >= max {
        return Err(NodeError::range_error("min must be less than max"));
    }

    let range = (max - min) as u64;
    let mut bytes = [0u8; 8];
    random_fill(&mut bytes)?;

    let random_value = u64::from_le_bytes(bytes);
    Ok(min + (random_value % range) as i64)
}

/// Create a hash
pub fn create_hash(algorithm: &str) -> Result<Hash> {
    Hash::new(algorithm)
}

/// Create an HMAC
pub fn create_hmac(algorithm: &str, key: &[u8]) -> Result<Hmac> {
    Hmac::new(algorithm, key)
}

/// Convenience function to hash data directly
pub fn hash_data(algorithm: &str, data: &[u8], encoding: Option<&str>) -> Result<String> {
    let mut hasher = Hash::new(algorithm)?;
    hasher.update(data);
    Ok(hasher.digest(encoding))
}

/// PBKDF2 key derivation
pub fn pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_length: usize,
    digest: &str,
) -> Result<Vec<u8>> {
    use ring::pbkdf2 as ring_pbkdf2;

    let algorithm = match digest.to_lowercase().as_str() {
        "sha1" => ring_pbkdf2::PBKDF2_HMAC_SHA1,
        "sha256" | "sha-256" => ring_pbkdf2::PBKDF2_HMAC_SHA256,
        "sha384" | "sha-384" => ring_pbkdf2::PBKDF2_HMAC_SHA384,
        "sha512" | "sha-512" => ring_pbkdf2::PBKDF2_HMAC_SHA512,
        _ => {
            return Err(NodeError::Crypto(format!(
                "Unsupported PBKDF2 digest: {}",
                digest
            )));
        }
    };

    let mut key = vec![0u8; key_length];
    ring_pbkdf2::derive(
        algorithm,
        std::num::NonZeroU32::new(iterations)
            .ok_or_else(|| NodeError::range_error("iterations must be > 0"))?,
        salt,
        password,
        &mut key,
    );

    Ok(key)
}

/// Check if timing-safe equal
pub fn timing_safe_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    ring::constant_time::verify_slices_are_equal(a, b).is_ok()
}

/// Scrypt key derivation (simplified - would need proper implementation)
pub fn scrypt(
    password: &[u8],
    salt: &[u8],
    n: u64,
    r: u32,
    p: u32,
    key_length: usize,
) -> Result<Vec<u8>> {
    // Note: ring doesn't have scrypt, would need another crate
    // For now, fall back to PBKDF2 with high iterations
    let iterations = (n * r as u64 * p as u64).min(u32::MAX as u64) as u32;
    pbkdf2(password, salt, iterations, key_length, "sha256")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let mut hash = Hash::new("sha256").unwrap();
        hash.update(b"hello world");
        let digest = hash.digest(Some("hex"));
        assert_eq!(
            digest,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hmac_sha256() {
        let mut hmac = Hmac::new("sha256", b"secret").unwrap();
        hmac.update(b"hello world");
        let digest = hmac.digest(Some("hex"));
        assert!(!digest.is_empty());
    }

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(32).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_random_uuid() {
        let uuid = random_uuid();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_random_int() {
        let val = random_int(0, 100).unwrap();
        assert!(val >= 0 && val < 100);
    }

    #[test]
    fn test_pbkdf2() {
        let key = pbkdf2(b"password", b"salt", 10000, 32, "sha256").unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_timing_safe_equal() {
        assert!(timing_safe_equal(b"hello", b"hello"));
        assert!(!timing_safe_equal(b"hello", b"world"));
        assert!(!timing_safe_equal(b"hello", b"hell"));
    }
}
