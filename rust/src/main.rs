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
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("flag '{flag}' requires a value"))
}

fn cache_root(p: &Parsed) -> PathBuf {
    PathBuf::from(
        p.cache_dir
            .clone()
            .unwrap_or_else(|| DEFAULT_CACHE.to_string()),
    )
}

fn base_dir(p: &Parsed) -> PathBuf {
    PathBuf::from(p.base_dir.clone().unwrap_or_else(|| ".".to_string()))
}

fn cmd_hash(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    let files = if p.positional.is_empty() {
        &p.inputs
    } else {
        &p.positional
    };
    if files.is_empty() {
        return Err("hash: expected one or more file paths".into());
    }
    for f in files {
        let (digest, size) = hash_file(Path::new(f)).map_err(|e| format!("{f}: {e}"))?;
        println!("{digest}  {size:>10}  {f}");
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_key(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    if p.inputs.is_empty() {
        return Err("key: expected at least one --input".into());
    }
    let base = base_dir(&p);
    let inputs = Store::hash_inputs(&base, &p.inputs).map_err(|e| e.to_string())?;
    let key = Manifest::compute_key(&inputs, &p.command);
    println!("{key}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_store(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    if p.inputs.is_empty() {
        return Err("store: expected at least one --input".into());
    }
    let base = base_dir(&p);
    let store = Store::open(cache_root(&p)).map_err(|e| e.to_string())?;
    let inputs = Store::hash_inputs(&base, &p.inputs).map_err(|e| e.to_string())?;
    let computed_key = Manifest::compute_key(&inputs, &p.command);
    // If the user supplied a key, honor it but warn on mismatch.
    let key = match &p.key {
        Some(k) if *k != computed_key => {
            eprintln!(
                "cairn: warning: supplied key {k} does not match computed key {computed_key}; using computed key"
            );
            computed_key
        }
        _ => computed_key,
    };
    let outputs = store
        .store_outputs(&base, &p.outputs)
        .map_err(|e| e.to_string())?;
    let manifest = Manifest {
        version: FORMAT_VERSION,
        producer: PRODUCER.to_string(),
        key: key.clone(),
        command: p.command.clone(),
        inputs,
        outputs,
    };
    store.put_manifest(&manifest).map_err(|e| e.to_string())?;
    println!("stored {} (outputs: {})", key, manifest.outputs.len());
    Ok(ExitCode::SUCCESS)
}

fn cmd_restore(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    let key = p.key.clone().ok_or("restore: --key is required")?;
    let store = Store::open(cache_root(&p)).map_err(|e| e.to_string())?;
    let Some(manifest) = store.get_manifest(&key).map_err(|e| e.to_string())? else {
        eprintln!("cairn: cache miss for key {key}");
        return Ok(ExitCode::from(2));
    };
    let out_dir = PathBuf::from(p.out_dir.clone().unwrap_or_else(|| ".".to_string()));
    let n = store
        .restore_outputs(&out_dir, &manifest)
        .map_err(|e| e.to_string())?;
    println!("restored {n} output(s) for key {key}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_verify(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    let key = p.key.clone().ok_or("verify: --key is required")?;
    let store = Store::open(cache_root(&p)).map_err(|e| e.to_string())?;
    let Some(manifest) = store.get_manifest(&key).map_err(|e| e.to_string())? else {
        return Err(format!("no manifest for key {key}"));
    };
    let report = store
        .verify_manifest(&manifest)
        .map_err(|e| e.to_string())?;
    println!(
        "key {key}: {} ok, {} missing, {} corrupt, key_matches={}",
        report.ok,
        report.missing.len(),
        report.corrupt.len(),
        report.key_matches
    );
    for m in &report.missing {
        println!("  MISSING {m}");
    }
    for c in &report.corrupt {
        println!("  CORRUPT {c}");
    }
    if report.is_healthy() {
        println!("OK: cache is healthy");
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("FAIL: cache integrity check failed");
        Ok(ExitCode::from(3))
    }
}

fn cmd_show(args: &[String]) -> Result<ExitCode, String> {
    let p = parse(args)?;
    let key = p.key.clone().ok_or("show: --key is required")?;
    let store = Store::open(cache_root(&p)).map_err(|e| e.to_string())?;
    match store.get_manifest(&key).map_err(|e| e.to_string())? {
        Some(m) => {
            println!("{}", m.to_json());
            Ok(ExitCode::SUCCESS)
        }
        None => {
            eprintln!("cairn: no manifest for key {key}");
            Ok(ExitCode::from(2))
        }
    }
}

fn print_usage() {
    eprintln!(
        "cairn {} — content-addressed build cache\n\
\n\
USAGE:\n\
    cairn <command> [options]\n\
\n\
COMMANDS:\n\
    hash <file>...                       Print content digests of files\n\
    key    --input F... [-- cmd...]      Compute a cache key from inputs\n\
    store  --input F... --output F... [--key K] [-- cmd...]\n\
                                         Store outputs and write a manifest\n\
    restore --key K [--out-dir DIR]      Restore outputs from cache\n\
    verify  --key K                      Verify store integrity for a key\n\
    show    --key K                      Print a manifest as JSON\n\
    version                              Print version\n\
\n\
GLOBAL OPTIONS:\n\
    --cache-dir DIR   Cache root (default: {DEFAULT_CACHE})\n\
    --base-dir  DIR   Base directory for --input/--output (default: .)\n\
    --                Everything after is treated as the command\n",
        env!("CARGO_PKG_VERSION")
    );
}
