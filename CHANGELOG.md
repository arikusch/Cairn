# Changelog

All notable changes to this project are documented here. The format is based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.1.0] - 2026-08-05

### Added

- `examples/demo.ps1` — the end-to-end walk (hash → MISS → HIT →
  cross-language restore/verify) now has a first-class Windows script
  alongside `examples/demo.sh`.
- README "Known limits of the survey" section stating what Cairn
  deliberately does not do (no GC, no locking, no dependency discovery).

### Changed

- The README is now a field guide to `docs/FORMAT.md` rather than a
  separate description of the format: every claim points at the spec.

## [4.3.0] - 2026-07-26

### Added

- "The performance ledger" guidance: how to measure miss vs hit cost for
  your own workload instead of trusting a headline number.
- Manifest and blob writes are fsynced before the atomic rename, so a crash
  cannot leave a half-written object where a reader might find it.

### Fixed

- `cairn verify` now reports missing and corrupt blobs by digest on separate
  lines, so a long list stays readable in narrow terminals.

## [3.2.0] - 2026-06-14

### Added

- "Cache-key anatomy" documentation: the exact NUL-delimited key stream,
  with the rationale for sorting inputs by path and folding every command
  argument into the key.
- Key-stability tests in both engines covering argument order, flags, and
  paths normalized to forward slashes.

### Changed

- Inputs are sorted by path **before** hashing in both implementations, so
  `-i b.c -i a.c` and `-i a.c -i b.c` produce one key.

## [2.4.0] - 2026-04-26

### Added

- `--base-dir` (Go wrapper and engine) to resolve `--input`/`--output`
  relative to a directory other than the process working directory.
- `--out-dir` for `cairn restore`.
- Canonical manifest field order is now guaranteed by construction in both
  writers and asserted in the test suites.

### Changed

- The manifest writer in the Go wrapper is hand-rolled (like the Rust one)
  instead of relying on map iteration order.

## [1.0.0] - 2026-02-08

### Added

- First stable release. The v1 format is frozen: on-disk layout, key stream,
  and canonical manifest JSON will not change without a format-version bump.
- Full verification path: `cairn verify --key K` checks blob presence, blob
  re-hashing, and key recomputation from the manifest's own inputs.

### Notes

- FNV-1a remains intentionally **non-cryptographic**; Cairn is a build
  cache, not a security boundary. See `docs/FORMAT.md` for the rationale.

## [0.9.0] - 2025-03-16

### Added

- Cross-language demo: the script builds both tools, shows a miss then a
  hit, and hands the manifest to the other tool to restore and verify.
- `Store::verify_manifest` / `VerifyManifest` returning a structured
  ok/missing/corrupt/key_matches report.

## [0.8.0] - 2024-04-07

### Added

- Deduplicated blob storage sharded by the first two hex characters of the
  digest (`objects/<aa>/<digest>`), shared by both tools.
- Bounded-memory hashing: files are streamed in 64 KiB chunks instead of
  being read whole.

## [0.7.0] - 2023-05-21

### Added

- Continuous integration across three jobs: Rust (fmt, clippy, test,
  release build), Go (vet, gofmt, test, build), and a cross-language interop
  job that runs the demo end to end.
- `cairn-run hash` and `cairn-run version` standalone subcommands.

## [0.6.0] - 2022-06-12

### Added

- `docs/FORMAT.md` — the normative specification of hashing, the cache-key
  stream, the manifest JSON, and the on-disk layout.
- `Makefile` targets (`build`, `test`, `fmt`, `vet`, `demo`, `clean`) and
  the scripted demo under `examples/`.

### Changed

- `cairn show` prints the manifest exactly as stored: canonical, compact,
  fixed key order.

## [0.5.0] - 2021-10-17

### Added

- **Go wrapper (`cairn-run`)**: wraps any command, restores declared
