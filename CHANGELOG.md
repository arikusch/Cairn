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

