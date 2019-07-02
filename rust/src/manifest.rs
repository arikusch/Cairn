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

/// A single captured output file.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputEntry {
    /// Logical path, using forward slashes.
    pub path: String,
    /// Content digest (canonical 16-char hex).
    pub digest: String,
    /// Size in bytes.
    pub size: u64,
}

/// A complete cache manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    /// Format version.
    pub version: u64,
    /// Tool that produced this manifest, e.g. "cairn-rust" or "cairn-go".
    pub producer: String,
    /// The overall cache key derived from inputs (+ optional command).
    pub key: String,
    /// The command associated with this entry, if any (empty for pure hashing).
    pub command: Vec<String>,
    /// Sorted input entries.
    pub inputs: Vec<InputEntry>,
    /// Sorted output entries.
    pub outputs: Vec<OutputEntry>,
}

impl Manifest {
    /// Compute the canonical cache key from a set of inputs and an optional
    /// command.
    ///
    /// The key is defined as the FNV-1a digest of a canonical byte stream:
    ///
    /// ```text
    /// "cairn-key\x00" version "\x00"
    /// for each input (sorted by path):
    ///     path "\x00" digest "\x00" size "\x00"
    /// "cmd\x00"
    /// for each command arg (in order):
    ///     arg "\x00"
    /// ```
    ///
    /// Inputs MUST be sorted by path before calling. The Go implementation
    /// reproduces this stream byte-for-byte.
    pub fn compute_key(inputs: &[InputEntry], command: &[String]) -> String {
        let mut h = Hasher::new();
        h.update(b"cairn-key\x00");
        h.update(FORMAT_VERSION.to_string().as_bytes());
        h.update(b"\x00");
        for input in inputs {
            h.update(input.path.as_bytes());
            h.update(b"\x00");
            h.update(input.digest.as_bytes());
            h.update(b"\x00");
            h.update(input.size.to_string().as_bytes());
            h.update(b"\x00");
        }
        h.update(b"cmd\x00");
        for arg in command {
            h.update(arg.as_bytes());
            h.update(b"\x00");
        }
        h.finalize_hex()
    }

    /// Serialize to canonical JSON.
    pub fn to_json(&self) -> String {
        let inputs = Json::Array(
            self.inputs
                .iter()
                .map(|i| {
                    json::object(vec![
                        ("path", Json::Str(i.path.clone())),
                        ("digest", Json::Str(i.digest.clone())),
                        ("size", Json::Uint(i.size)),
                    ])
                })
                .collect(),
        );
        let outputs = Json::Array(
            self.outputs
                .iter()
