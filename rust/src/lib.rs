//! Cairn: a universal, content-addressed build cache.
//!
//! This crate exposes the reusable engine (hashing, manifest format, and the
//! content-addressed store) plus a CLI binary (`cairn`). The manifest format
//! and hashing scheme are documented in `docs/FORMAT.md` and are reproduced
//! exactly by the companion Go wrapper so the two tools interoperate.
//!
//! # Modules
//!
//! - [`hash`]: deterministic, non-cryptographic FNV-1a content hashing.
//! - [`json`]: a tiny std-only JSON reader/writer for canonical manifests.
//! - [`manifest`]: the cross-language cache-entry format.
//! - [`store`]: the on-disk content-addressed object/manifest store.

pub mod hash;
pub mod json;
pub mod manifest;
