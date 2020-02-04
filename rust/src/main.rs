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
        Ok(code) => code,
        Err(e) => {
            eprintln!("cairn: error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some((cmd, rest)) = args.split_first() else {
        print_usage();
        return Ok(ExitCode::FAILURE);
    };
    match cmd.as_str() {
        "hash" => cmd_hash(rest),
        "key" => cmd_key(rest),
        "store" => cmd_store(rest),
        "restore" => cmd_restore(rest),
        "verify" => cmd_verify(rest),
        "show" => cmd_show(rest),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        "version" | "--version" | "-V" => {
            println!(
                "cairn {} (format v{})",
                env!("CARGO_PKG_VERSION"),
                FORMAT_VERSION
            );
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand '{other}' (try 'cairn help')")),
    }
}

/// A minimal flag parser tailored to Cairn's options. Recognizes repeated
/// `--input`/`--output`, single-valued `--key`/`--cache-dir`/`--out-dir`, and
/// a `--` separator after which everything is treated as a command.
#[derive(Default)]
struct Parsed {
    inputs: Vec<String>,
    outputs: Vec<String>,
    key: Option<String>,
    cache_dir: Option<String>,
    out_dir: Option<String>,
    base_dir: Option<String>,
    command: Vec<String>,
    positional: Vec<String>,
}

fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut p = Parsed::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--" => {
                p.command = args[i + 1..].to_vec();
                break;
            }
            "--input" | "-i" => {
                p.inputs.push(take_value(args, &mut i, a)?);
            }
            "--output" | "-o" => {
                p.outputs.push(take_value(args, &mut i, a)?);
            }
            "--key" | "-k" => {
                p.key = Some(take_value(args, &mut i, a)?);
            }
            "--cache-dir" => {
                p.cache_dir = Some(take_value(args, &mut i, a)?);
            }
            "--out-dir" => {
                p.out_dir = Some(take_value(args, &mut i, a)?);
            }
            "--base-dir" => {
                p.base_dir = Some(take_value(args, &mut i, a)?);
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag '{other}'"));
            }
            _ => p.positional.push(a.clone()),
        }
        i += 1;
    }
    Ok(p)
}

fn take_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
