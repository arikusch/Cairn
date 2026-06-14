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
                .map(|o| {
                    json::object(vec![
                        ("path", Json::Str(o.path.clone())),
                        ("digest", Json::Str(o.digest.clone())),
                        ("size", Json::Uint(o.size)),
                    ])
                })
                .collect(),
        );
        let command = Json::Array(self.command.iter().map(|c| Json::Str(c.clone())).collect());
        let doc = json::object(vec![
            ("version", Json::Uint(self.version)),
            ("producer", Json::Str(self.producer.clone())),
            ("key", Json::Str(self.key.clone())),
            ("command", command),
            ("inputs", inputs),
            ("outputs", outputs),
        ]);
        doc.encode()
    }

    /// Parse a manifest from canonical (or any well-formed) JSON.
    pub fn from_json(text: &str) -> Result<Manifest, String> {
        let doc = json::parse(text).map_err(|e| e.to_string())?;
        let version = doc
            .get("version")
            .and_then(Json::as_uint)
            .ok_or("missing 'version'")?;
        let producer = doc
            .get("producer")
            .and_then(Json::as_str)
            .ok_or("missing 'producer'")?
            .to_string();
        let key = doc
            .get("key")
            .and_then(Json::as_str)
            .ok_or("missing 'key'")?
            .to_string();
        let command = doc
            .get("command")
            .and_then(Json::as_array)
            .ok_or("missing 'command'")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or("command arg not a string")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let inputs = parse_entries(doc.get("inputs").ok_or("missing 'inputs'")?)?
            .into_iter()
            .map(|(path, digest, size)| InputEntry { path, digest, size })
            .collect();
        let outputs = parse_entries(doc.get("outputs").ok_or("missing 'outputs'")?)?
            .into_iter()
            .map(|(path, digest, size)| OutputEntry { path, digest, size })
            .collect();
        Ok(Manifest {
            version,
            producer,
            key,
            command,
            inputs,
            outputs,
        })
    }
}

fn parse_entries(value: &Json) -> Result<Vec<(String, String, u64)>, String> {
    let arr = value.as_array().ok_or("expected array of entries")?;
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let path = entry
            .get("path")
            .and_then(Json::as_str)
            .ok_or("entry missing 'path'")?
            .to_string();
        let digest = entry
            .get("digest")
            .and_then(Json::as_str)
            .ok_or("entry missing 'digest'")?
            .to_string();
        let size = entry
            .get("size")
            .and_then(Json::as_uint)
            .ok_or("entry missing 'size'")?;
        out.push((path, digest, size));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        let inputs = vec![
            InputEntry {
                path: "src/a.txt".into(),
                digest: "af63dc4c8601ec8c".into(),
                size: 1,
            },
            InputEntry {
                path: "src/b.txt".into(),
                digest: "85944171f73967e8".into(),
                size: 6,
            },
        ];
        let command = vec!["cc".to_string(), "-c".to_string()];
        let key = Manifest::compute_key(&inputs, &command);
        Manifest {
            version: FORMAT_VERSION,
            producer: "cairn-rust".into(),
            key,
            command,
            inputs,
            outputs: vec![OutputEntry {
                path: "out/a.o".into(),
                digest: "cbf29ce484222325".into(),
                size: 0,
            }],
        }
    }

    #[test]
    fn json_roundtrip() {
        let m = sample();
        let text = m.to_json();
        let back = Manifest::from_json(&text).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn key_is_stable() {
        let m = sample();
        let key2 = Manifest::compute_key(&m.inputs, &m.command);
        assert_eq!(m.key, key2);
    }

    #[test]
    fn key_changes_with_command() {
        let m = sample();
        let other = Manifest::compute_key(&m.inputs, &["cc".to_string()]);
        assert_ne!(m.key, other);
    }
}
