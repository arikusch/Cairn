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
