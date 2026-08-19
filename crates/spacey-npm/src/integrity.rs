//! Integrity verification for packages.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Integrity checker for verifying package hashes.
#[derive(Clone)]
pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Create a new integrity checker.
    pub fn new() -> Self {
        Self
    }

    /// Verify an integrity hash (SRI format).
    ///
    /// Supports sha256, sha384, sha512 hashes in the format:
    /// `sha512-BASE64HASH`
    pub fn verify_integrity(&self, data: &[u8], integrity: &str) -> bool {
        // Parse integrity string (may contain multiple hashes separated by space)
        for hash_str in integrity.split_whitespace() {
            if let Some((algo, expected_hash)) = hash_str.split_once('-') {
                let computed = match algo {
                    "sha256" => {
                        let hash = Sha256::digest(data);
                        BASE64.encode(hash)
                    }
                    "sha384" => {
                        let hash = Sha384::digest(data);
                        BASE64.encode(hash)
                    }
                    "sha512" => {
                        let hash = Sha512::digest(data);
                        BASE64.encode(hash)
                    }
                    _ => continue,
                };

                if computed == expected_hash {
                    return true;
                }
            }
        }

        false
    }

    /// Verify a SHA-1 hash (legacy format).
    pub fn verify_shasum(&self, data: &[u8], expected: &str) -> bool {
        let computed = self.compute_shasum(data);
        computed.eq_ignore_ascii_case(expected)
    }

    /// Compute SHA-512 integrity hash.
    pub fn compute_integrity(&self, data: &[u8]) -> String {
        let hash = Sha512::digest(data);
        format!("sha512-{}", BASE64.encode(hash))
    }

    /// Compute SHA-1 hash.
    pub fn compute_shasum(&self, data: &[u8]) -> String {
        use sha1::Digest as Sha1Digest;
        let hash = Sha1::digest(data);
        ::hex::encode(hash)
    }

    /// Compute SHA-256 integrity hash.
    pub fn compute_sha256(&self, data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        format!("sha256-{}", BASE64.encode(hash))
    }
}

impl Default for IntegrityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha512_integrity() {
        let checker = IntegrityChecker::new();
        let data = b"hello world";

        let integrity = checker.compute_integrity(data);
        assert!(integrity.starts_with("sha512-"));
        assert!(checker.verify_integrity(data, &integrity));
    }

    #[test]
    fn test_shasum() {
        let checker = IntegrityChecker::new();
        let data = b"hello world";

        let shasum = checker.compute_shasum(data);
        assert!(checker.verify_shasum(data, &shasum));
    }
}
