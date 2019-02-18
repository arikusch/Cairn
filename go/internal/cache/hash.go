// Package cache implements the Cairn content-addressed cache format in Go.
//
// It is a faithful, standard-library-only port of the Rust engine: the same
// FNV-1a 64-bit hashing scheme, the same canonical cache-key stream, and the
// same manifest JSON layout. A manifest written by this package can be read
// and restored by the Rust `cairn` binary and vice versa.
//
// Security note: FNV-1a is NOT cryptographic. See docs/FORMAT.md.
package cache

import (
	"fmt"
	"io"
	"os"
)

const (
	fnvOffsetBasis uint64 = 0xcbf29ce484222325
	fnvPrime       uint64 = 0x00000100000001b3
)

// Hasher is a streaming FNV-1a 64-bit hasher, matching rust/src/hash.rs.
type Hasher struct {
	state uint64
}

// NewHasher returns a hasher seeded with the FNV offset basis.
