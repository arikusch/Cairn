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
	baseDir  string
	inputs   []string
	outputs  []string
	force    bool
	verbose  bool
	command  []string
}

func main() {
	args := os.Args[1:]
	if len(args) == 0 {
		usage(os.Stderr)
		os.Exit(exitError)
	}

	// Standalone subcommands that do not take a `--` command.
	switch args[0] {
	case "version", "--version", "-V":
		fmt.Printf("cairn-run %s (format v%d)\n", version, cache.FormatVersion)
		return
	case "help", "-h", "--help":
		usage(os.Stdout)
		return
	case "hash":
		if err := cmdHash(args[1:]); err != nil {
			fmt.Fprintf(os.Stderr, "cairn-run: error: %v\n", err)
			os.Exit(exitError)
		}
		return
	}

	opts, err := parseOptions(args)
	if err != nil {
		fmt.Fprintf(os.Stderr, "cairn-run: error: %v\n", err)
		os.Exit(exitError)
	}
	code, err := runWrapped(opts)
	if err != nil {
		fmt.Fprintf(os.Stderr, "cairn-run: error: %v\n", err)
		os.Exit(exitError)
	}
	os.Exit(code)
}

func parseOptions(args []string) (options, error) {
	opts := options{cacheDir: defaultCache, baseDir: "."}
	i := 0
	for i < len(args) {
		a := args[i]
		switch a {
		case "--":
			opts.command = args[i+1:]
			i = len(args)
			continue
		case "--cache-dir":
			v, err := takeValue(args, &i, a)
			if err != nil {
				return opts, err
			}
			opts.cacheDir = v
		case "--base-dir":
			v, err := takeValue(args, &i, a)
			if err != nil {
				return opts, err
			}
			opts.baseDir = v
		case "--input", "-i":
			v, err := takeValue(args, &i, a)
			if err != nil {
				return opts, err
			}
			opts.inputs = append(opts.inputs, v)
		case "--output", "-o":
			v, err := takeValue(args, &i, a)
			if err != nil {
				return opts, err
			}
			opts.outputs = append(opts.outputs, v)
		case "--force":
			opts.force = true
		case "--verbose":
			opts.verbose = true
		default:
			return opts, fmt.Errorf("unknown flag %q", a)
		}
		i++
	}
	if len(opts.command) == 0 {
		return opts, fmt.Errorf("no command given (expected '-- <command> [args...]')")
	}
	return opts, nil
}

func takeValue(args []string, i *int, flag string) (string, error) {
	*i++
	if *i >= len(args) {
		return "", fmt.Errorf("flag %q requires a value", flag)
	}
	return args[*i], nil
}

// runWrapped implements the cache-aware execution flow and returns the process
// exit code to propagate.
func runWrapped(opts options) (int, error) {
	store, err := cache.OpenStore(opts.cacheDir)
	if err != nil {
		return exitError, err
	}
	inputs, err := cache.HashInputs(opts.baseDir, opts.inputs)
	if err != nil {
		return exitError, err
	}
	key := cache.ComputeKey(inputs, opts.command)

	if !opts.force {
		manifest, err := store.GetManifest(key)
		if err != nil {
			return exitError, err
		}
		if manifest != nil {
			n, err := store.RestoreOutputs(opts.baseDir, manifest)
			if err == nil {
				if opts.verbose {
					fmt.Fprintf(os.Stderr, "cairn-run: cache HIT %s (restored %d output(s))\n", key, n)
				}
				return 0, nil
			}
			// A manifest exists but restore failed (e.g. pruned blobs): fall
			// through to re-run rather than fail outright.
			if opts.verbose {
				fmt.Fprintf(os.Stderr, "cairn-run: manifest present but restore failed (%v); re-running\n", err)
			}
		} else if opts.verbose {
			fmt.Fprintf(os.Stderr, "cairn-run: cache MISS %s\n", key)
		}
	} else if opts.verbose {
		fmt.Fprintf(os.Stderr, "cairn-run: --force set, running command\n")
	}

	// Cache miss (or forced): execute the command.
	exitCode, err := execCommand(opts)
	if err != nil {
		return exitError, err
	}
	if exitCode != 0 {
		// Do not cache failed builds.
		if opts.verbose {
			fmt.Fprintf(os.Stderr, "cairn-run: command exited %d; not caching\n", exitCode)
		}
		return exitCode, nil
	}

	outputs, err := store.StoreOutputs(opts.baseDir, opts.outputs)
	if err != nil {
		return exitError, err
	}
	manifest := &cache.Manifest{
		Version:  cache.FormatVersion,
		Producer: producer,
		Key:      key,
		Command:  opts.command,
		Inputs:   inputs,
		Outputs:  outputs,
	}
	if err := store.PutManifest(manifest); err != nil {
		return exitError, err
	}
	if opts.verbose {
		fmt.Fprintf(os.Stderr, "cairn-run: stored manifest %s (%d output(s))\n", key, len(outputs))
	}
	return 0, nil
}

func execCommand(opts options) (int, error) {
	cmd := exec.Command(opts.command[0], opts.command[1:]...)
	cmd.Dir = opts.baseDir
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	err := cmd.Run()
	if err == nil {
		return 0, nil
