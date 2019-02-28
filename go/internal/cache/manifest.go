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
