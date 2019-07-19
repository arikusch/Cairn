//! The on-disk content-addressed store.
//!
//! Layout under the cache root (default `.cairn-cache`):
//!
//! ```text
//! <root>/
//!   objects/<aa>/<digest>       content-addressed blobs (aa = first 2 hex chars)
//!   manifests/<key>.json        manifests keyed by cache key
//! ```
//!
//! Blobs are deduplicated: identical content is stored once regardless of how
//! many manifests reference it. The same layout is understood by the Go
//! wrapper, so a manifest written by one tool can be restored by the other.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::hash::Hasher;
use crate::manifest::{InputEntry, Manifest, OutputEntry};

/// Read a whole file, streaming it through the hasher.
///
/// Returns the canonical digest and the byte length. Reads in fixed chunks so
/// arbitrarily large files use bounded memory.
pub fn hash_file(path: &Path) -> std::io::Result<(String, u64)> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total: u64 = 0;
    loop {
