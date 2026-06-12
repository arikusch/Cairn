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
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total += n as u64;
    }
    Ok((hasher.finalize_hex(), total))
}

/// The content-addressed store rooted at a directory.
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open (creating if needed) a store at `root`.
    pub fn open(root: impl Into<PathBuf>) -> std::io::Result<Store> {
        let root = root.into();
        fs::create_dir_all(root.join("objects"))?;
        fs::create_dir_all(root.join("manifests"))?;
        Ok(Store { root })
    }

    /// The path where a blob with the given digest lives.
    fn object_path(&self, digest: &str) -> PathBuf {
        let shard = &digest[..2.min(digest.len())];
        self.root.join("objects").join(shard).join(digest)
    }

    /// The path where a manifest with the given key lives.
    fn manifest_path(&self, key: &str) -> PathBuf {
        self.root.join("manifests").join(format!("{key}.json"))
    }

    /// Store a blob by its content, returning its digest. Idempotent: writing
    /// content that already exists is a no-op.
    pub fn put_blob(&self, content: &[u8]) -> std::io::Result<String> {
        let mut hasher = Hasher::new();
        hasher.update(content);
        let digest = hasher.finalize_hex();
        let dest = self.object_path(&digest);
        if dest.exists() {
            return Ok(digest);
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        // Write to a temp file then rename for atomicity.
        let tmp = dest.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(content)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &dest)?;
        Ok(digest)
    }

    /// Store a file's contents as a blob, returning digest and size.
    pub fn put_file(&self, path: &Path) -> std::io::Result<(String, u64)> {
        let content = fs::read(path)?;
        let size = content.len() as u64;
        let digest = self.put_blob(&content)?;
        Ok((digest, size))
    }

    /// Read a blob by digest.
    pub fn get_blob(&self, digest: &str) -> std::io::Result<Vec<u8>> {
        fs::read(self.object_path(digest))
    }

    /// True if a blob with the given digest is present.
    pub fn has_blob(&self, digest: &str) -> bool {
        self.object_path(digest).exists()
    }

    /// Persist a manifest under its key.
    pub fn put_manifest(&self, manifest: &Manifest) -> std::io::Result<()> {
        let dest = self.manifest_path(&manifest.key);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = dest.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(manifest.to_json().as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &dest)?;
        Ok(())
    }

    /// Load a manifest by key, if present.
    pub fn get_manifest(&self, key: &str) -> std::io::Result<Option<Manifest>> {
        let path = self.manifest_path(key);
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(path)?;
        Manifest::from_json(&text)
            .map(Some)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// True if a manifest exists for the key.
    pub fn has_manifest(&self, key: &str) -> bool {
        self.manifest_path(key).exists()
    }

    /// Hash a set of input files (relative to `base`) into sorted input
    /// entries. Paths are normalized to forward slashes for cross-platform
    /// stability.
    pub fn hash_inputs(base: &Path, paths: &[String]) -> std::io::Result<Vec<InputEntry>> {
        let mut entries = Vec::with_capacity(paths.len());
        for p in paths {
            let full = base.join(p);
            let (digest, size) = hash_file(&full)?;
            entries.push(InputEntry {
                path: normalize_path(p),
                digest,
                size,
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    /// Store output files as blobs and produce sorted output entries.
    pub fn store_outputs(
        &self,
        base: &Path,
        paths: &[String],
    ) -> std::io::Result<Vec<OutputEntry>> {
        let mut entries = Vec::with_capacity(paths.len());
        for p in paths {
            let full = base.join(p);
            let (digest, size) = self.put_file(&full)?;
            entries.push(OutputEntry {
                path: normalize_path(p),
                digest,
                size,
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    /// Restore all outputs from a manifest into `base`. Returns the number of
    /// files written. Fails if any referenced blob is missing.
    pub fn restore_outputs(&self, base: &Path, manifest: &Manifest) -> std::io::Result<usize> {
        for out in &manifest.outputs {
            if !self.has_blob(&out.digest) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("missing blob {} for output {}", out.digest, out.path),
                ));
            }
        }
        let mut written = 0;
        for out in &manifest.outputs {
            let content = self.get_blob(&out.digest)?;
            let dest = base.join(&out.path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, &content)?;
            written += 1;
        }
        Ok(written)
    }

    /// Verify store integrity for a manifest: every referenced blob must exist
    /// and re-hash to its recorded digest. Returns a report.
    pub fn verify_manifest(&self, manifest: &Manifest) -> std::io::Result<VerifyReport> {
        let mut report = VerifyReport::default();
        for out in &manifest.outputs {
            match self.get_blob(&out.digest) {
                Ok(content) => {
                    let mut h = Hasher::new();
                    h.update(&content);
                    if h.finalize_hex() == out.digest {
                        report.ok += 1;
                    } else {
                        report.corrupt.push(out.digest.clone());
                    }
                }
                Err(_) => report.missing.push(out.digest.clone()),
            }
        }
        // Recompute the key from inputs + command and confirm it matches.
        let recomputed = Manifest::compute_key(&manifest.inputs, &manifest.command);
        report.key_matches = recomputed == manifest.key;
        Ok(report)
    }
}

/// Result of verifying a manifest against the store.
#[derive(Debug, Default)]
pub struct VerifyReport {
    /// Number of outputs present and matching their digest.
    pub ok: usize,
    /// Digests of outputs whose blob is missing.
    pub missing: Vec<String>,
    /// Digests of outputs whose blob content no longer matches.
    pub corrupt: Vec<String>,
    /// Whether the recomputed key matches the stored key.
    pub key_matches: bool,
}

impl VerifyReport {
    /// True when nothing is missing or corrupt and the key checks out.
    pub fn is_healthy(&self) -> bool {
        self.missing.is_empty() && self.corrupt.is_empty() && self.key_matches
    }
}

/// Normalize a path string to forward slashes for cross-platform manifests.
pub fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::FORMAT_VERSION;

    fn temp_dir(tag: &str) -> PathBuf {
        let mut d = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        d.push(format!("cairn-test-{tag}-{nanos}"));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn blob_roundtrip_and_dedup() {
        let dir = temp_dir("blob");
        let store = Store::open(dir.join("cache")).unwrap();
        let d1 = store.put_blob(b"hello").unwrap();
        let d2 = store.put_blob(b"hello").unwrap();
        assert_eq!(d1, d2);
        assert!(store.has_blob(&d1));
        assert_eq!(store.get_blob(&d1).unwrap(), b"hello");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn store_restore_and_verify() {
        let dir = temp_dir("restore");
        let work = dir.join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("in.txt"), b"foobar").unwrap();
        fs::write(work.join("out.txt"), b"result-bytes").unwrap();

        let store = Store::open(dir.join("cache")).unwrap();
        let inputs = Store::hash_inputs(&work, &["in.txt".into()]).unwrap();
        let command = vec!["build".to_string()];
        let key = Manifest::compute_key(&inputs, &command);
        let outputs = store.store_outputs(&work, &["out.txt".into()]).unwrap();
        let manifest = Manifest {
            version: FORMAT_VERSION,
            producer: "cairn-rust".into(),
            key: key.clone(),
            command,
            inputs,
            outputs,
        };
        store.put_manifest(&manifest).unwrap();

        // Delete the output, then restore it from cache.
        fs::remove_file(work.join("out.txt")).unwrap();
        let loaded = store.get_manifest(&key).unwrap().unwrap();
        let n = store.restore_outputs(&work, &loaded).unwrap();
        assert_eq!(n, 1);
        assert_eq!(fs::read(work.join("out.txt")).unwrap(), b"result-bytes");

        let report = store.verify_manifest(&loaded).unwrap();
        assert!(report.is_healthy(), "report: {report:?}");
        fs::remove_dir_all(&dir).ok();
    }
}
