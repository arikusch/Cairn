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
	for _, in := range inputs {
		h.Update([]byte(in.Path))
		h.Update([]byte{0})
		h.Update([]byte(in.Digest))
		h.Update([]byte{0})
		h.Update([]byte(strconv.FormatUint(in.Size, 10)))
		h.Update([]byte{0})
	}
	h.Update([]byte("cmd\x00"))
	for _, arg := range command {
		h.Update([]byte(arg))
		h.Update([]byte{0})
	}
	return h.Hex()
}

// SortEntries sorts entries by path in place for deterministic keys.
func SortEntries(entries []Entry) {
	sort.Slice(entries, func(i, j int) bool { return entries[i].Path < entries[j].Path })
}

// NormalizePath converts backslashes to forward slashes for portable manifests.
func NormalizePath(p string) string {
