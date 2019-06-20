//! The Cairn manifest: the canonical, cross-language description of a cache
//! entry.
//!
//! A manifest records the inputs that were hashed to produce a cache key, the
//! outputs that were captured, and provenance metadata. Both the Rust engine
//! and the Go wrapper serialize manifests with identical field names and an
//! identical canonicalization scheme so the two tools are fully
//! interoperable. The wire format is documented in `docs/FORMAT.md`.

use crate::hash::Hasher;
use crate::json::{self, Json};

/// Current manifest format version. Bump on incompatible changes.
pub const FORMAT_VERSION: u64 = 1;

/// A single hashed input file.
#[derive(Clone, Debug, PartialEq)]
pub struct InputEntry {
    /// Logical path (as supplied on the command line), using forward slashes.
    pub path: String,
    /// Content digest (canonical 16-char hex).
    pub digest: String,
    /// Size in bytes.
    pub size: u64,
}
