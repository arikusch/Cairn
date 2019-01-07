//! Deterministic, non-cryptographic content hashing.
//!
//! Cairn uses the 64-bit FNV-1a hash to produce content addresses. FNV-1a is
//! fast, dependency-free, and fully deterministic across platforms and
//! languages, which makes it ideal for a build cache where the only
//! requirement is that identical bytes map to identical addresses.
//!
//! # Security note
//!
