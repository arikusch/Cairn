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

