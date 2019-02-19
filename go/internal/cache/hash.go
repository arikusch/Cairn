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
func NewHasher() *Hasher {
	return &Hasher{state: fnvOffsetBasis}
}

// Update absorbs bytes into the running digest.
func (h *Hasher) Update(b []byte) {
	state := h.state
	for _, c := range b {
		state ^= uint64(c)
		state *= fnvPrime
	}
	h.state = state
}

// Sum returns the raw 64-bit digest.
func (h *Hasher) Sum() uint64 { return h.state }

// Hex returns the canonical 16-char lowercase hex digest.
func (h *Hasher) Hex() string { return fmt.Sprintf("%016x", h.state) }

// HashBytes hashes a byte slice and returns the canonical hex digest.
func HashBytes(b []byte) string {
	h := NewHasher()
	h.Update(b)
	return h.Hex()
}

// HashFile streams a file through the hasher, returning digest and size.
func HashFile(path string) (string, uint64, error) {
	f, err := os.Open(path)
	if err != nil {
		return "", 0, err
	}
	defer f.Close()

	h := NewHasher()
	buf := make([]byte, 64*1024)
	var total uint64
	for {
		n, err := f.Read(buf)
