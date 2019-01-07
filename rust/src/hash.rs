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
