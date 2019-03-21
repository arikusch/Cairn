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
	return strings.ReplaceAll(p, "\\", "/")
}

// ToJSON serializes the manifest to canonical, compact JSON with fields in the
// exact order the Rust engine emits: version, producer, key, command, inputs,
// outputs.
func (m *Manifest) ToJSON() string {
	var b strings.Builder
	b.WriteByte('{')
	writeKey(&b, "version")
	b.WriteString(strconv.FormatUint(m.Version, 10))
	b.WriteByte(',')
	writeKey(&b, "producer")
	writeJSONString(&b, m.Producer)
	b.WriteByte(',')
	writeKey(&b, "key")
	writeJSONString(&b, m.Key)
	b.WriteByte(',')
	writeKey(&b, "command")
	writeStringArray(&b, m.Command)
	b.WriteByte(',')
	writeKey(&b, "inputs")
	writeEntryArray(&b, m.Inputs)
	b.WriteByte(',')
	writeKey(&b, "outputs")
	writeEntryArray(&b, m.Outputs)
	b.WriteByte('}')
	return b.String()
}

func writeKey(b *strings.Builder, key string) {
	writeJSONString(b, key)
	b.WriteByte(':')
}

func writeStringArray(b *strings.Builder, items []string) {
	b.WriteByte('[')
	for i, it := range items {
		if i > 0 {
			b.WriteByte(',')
		}
		writeJSONString(b, it)
	}
	b.WriteByte(']')
}

func writeEntryArray(b *strings.Builder, entries []Entry) {
	b.WriteByte('[')
	for i, e := range entries {
		if i > 0 {
			b.WriteByte(',')
		}
		b.WriteByte('{')
		writeKey(b, "path")
		writeJSONString(b, e.Path)
		b.WriteByte(',')
		writeKey(b, "digest")
		writeJSONString(b, e.Digest)
		b.WriteByte(',')
		writeKey(b, "size")
		b.WriteString(strconv.FormatUint(e.Size, 10))
		b.WriteByte('}')
	}
	b.WriteByte(']')
}

// writeJSONString writes a JSON string literal with the same escaping rules as
// the Rust writer (RFC 8259, lowercase \u for control chars).
