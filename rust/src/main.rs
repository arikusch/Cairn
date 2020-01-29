//! `cairn` — the content-addressed build cache CLI.
//!
//! Subcommands:
//!
//! ```text
//! cairn hash <file>...                       print digests of files
//! cairn key   --input F... [-- cmd...]       compute a cache key
//! cairn store --key K --input F... --output F... [-- cmd...]
//!                                            store outputs + manifest
//! cairn restore --key K [--out-dir DIR]      restore outputs from cache
//! cairn verify  --key K                      verify store integrity for a key
//! cairn show    --key K                      print a manifest as JSON
//! ```
//!
//! Global option: `--cache-dir DIR` (default `.cairn-cache`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cairn::manifest::{Manifest, FORMAT_VERSION};
use cairn::store::{hash_file, Store};

const DEFAULT_CACHE: &str = ".cairn-cache";
const PRODUCER: &str = "cairn-rust";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
