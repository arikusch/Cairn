// Package cache implements the Cairn content-addressed cache format in Go.
//
// It is a faithful, standard-library-only port of the Rust engine: the same
// FNV-1a 64-bit hashing scheme, the same canonical cache-key stream, and the
// same manifest JSON layout. A manifest written by this package can be read
// and restored by the Rust `cairn` binary and vice versa.
//
// Security note: FNV-1a is NOT cryptographic. See docs/FORMAT.md.
