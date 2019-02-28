package cache

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
)

// FormatVersion is the manifest format version, matching the Rust engine.
const FormatVersion uint64 = 1

// Entry is a hashed input or captured output file.
type Entry struct {
	Path   string
	Digest string
	Size   uint64
}

// Manifest is the cross-language cache-entry description.
type Manifest struct {
	Version  uint64
	Producer string
	Key      string
	Command  []string
	Inputs   []Entry
	Outputs  []Entry
}

// ComputeKey derives the canonical cache key from inputs and an optional
// command. Inputs MUST be sorted by path first (SortEntries does this). The
// byte stream is identical to Manifest::compute_key in rust/src/manifest.rs.
func ComputeKey(inputs []Entry, command []string) string {
	h := NewHasher()
	h.Update([]byte("cairn-key\x00"))
	h.Update([]byte(strconv.FormatUint(FormatVersion, 10)))
	h.Update([]byte{0})
