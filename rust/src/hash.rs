//! Deterministic, non-cryptographic content hashing.
//!
//! Cairn uses the 64-bit FNV-1a hash to produce content addresses. FNV-1a is
//! fast, dependency-free, and fully deterministic across platforms and
//! languages, which makes it ideal for a build cache where the only
//! requirement is that identical bytes map to identical addresses.
//!
//! # Security note
//!
//! FNV-1a is **NOT** a cryptographic hash. It provides no collision or
//! pre-image resistance against a malicious adversary. Cairn is a build cache,
//! not a security boundary: do not rely on these digests to detect deliberate
//! tampering. See `docs/FORMAT.md` for the full rationale.

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Streaming FNV-1a 64-bit hasher.
///
/// Feed bytes with [`Hasher::update`] and read the final digest with
/// [`Hasher::finalize`] (numeric) or [`Hasher::finalize_hex`] (canonical
/// 16-character lowercase hex string).
#[derive(Clone, Copy, Debug)]
pub struct Hasher {
    state: u64,
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    /// Create a fresh hasher seeded with the FNV offset basis.
    pub fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    /// Absorb a chunk of bytes into the running digest.
    pub fn update(&mut self, bytes: &[u8]) {
        let mut state = self.state;
        for &byte in bytes {
            state ^= u64::from(byte);
            state = state.wrapping_mul(FNV_PRIME);
        }
        self.state = state;
    }

    /// Return the raw 64-bit digest of everything absorbed so far.
    pub fn finalize(&self) -> u64 {
        self.state
    }

    /// Return the canonical digest: a zero-padded, lowercase, 16-char hex
    /// string. This is the exact form Cairn uses as a content address and the
    /// form the Go implementation must reproduce byte-for-byte.
    pub fn finalize_hex(&self) -> String {
        format!("{:016x}", self.state)
    }
}

/// Convenience: hash a byte slice and return the canonical hex digest.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(bytes);
    h.finalize_hex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        // Empty input hashes to the offset basis.
        assert_eq!(hash_bytes(b""), "cbf29ce484222325");
        // Well-known FNV-1a 64 test vector for "a".
        assert_eq!(hash_bytes(b"a"), "af63dc4c8601ec8c");
        // "foobar" is a classic FNV reference vector.
        assert_eq!(hash_bytes(b"foobar"), "85944171f73967e8");
    }

    #[test]
    fn streaming_matches_oneshot() {
        let mut h = Hasher::new();
        h.update(b"foo");
        h.update(b"bar");
        assert_eq!(h.finalize_hex(), hash_bytes(b"foobar"));
    }
}
