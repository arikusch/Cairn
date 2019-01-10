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
