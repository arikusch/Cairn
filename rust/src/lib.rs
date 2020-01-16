//! Cairn: a universal, content-addressed build cache.
//!
//! This crate exposes the reusable engine (hashing, manifest format, and the
//! content-addressed store) plus a CLI binary (`cairn`). The manifest format
//! and hashing scheme are documented in `docs/FORMAT.md` and are reproduced
