// Command cairn-run is the Cairn command-execution wrapper.
//
// It computes a cache key from declared input files and the command line. On a
// cache hit it restores the recorded outputs instead of re-running the
// command; on a miss it runs the command, captures the declared outputs into
// the content-addressed store, and writes a manifest. The store layout and
// manifest format are byte-compatible with the Rust `cairn` engine.
//
// Usage:
//
//	cairn-run [options] -- <command> [args...]
//
// Options:
//
//	--cache-dir DIR   cache root (default .cairn-cache)
//	--base-dir  DIR   base directory for inputs/outputs (default .)
//	--input     F     declare an input file (repeatable)
//	--output    F     declare an output file (repeatable)
//	--force           always run; still records the manifest
//	--verbose         print cache decisions
//
// Standalone subcommands (no `--`):
//
//	cairn-run hash <file>...    print digests
//	cairn-run version           print version
package main

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/jfrost4/Cairn/go/internal/cache"
)

const (
	version       = "5.1.0"
	defaultCache  = ".cairn-cache"
	producer      = "cairn-go"
	exitError     = 1
	exitCacheMiss = 2
)

type options struct {
	cacheDir string
