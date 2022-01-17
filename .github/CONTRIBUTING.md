# Contributing to Cairn

Thanks for your interest in Cairn. This document describes how to propose
changes that fit the project's constraints.

## Ground rules

Cairn is deliberately small and boring. Two rules decide whether a change
can be merged:

1. **Standard libraries only.** The Rust engine uses no third-party crates
   and the Go wrapper uses no modules beyond the toolchain. A pull request
   that adds a dependency needs a very strong argument and will usually be
   asked to reimplement the piece it needs.
2. **Both tools, one format.** Any change that touches hashing, the cache
   key, the manifest JSON, or the on-disk layout must be made in **both**
   implementations, must be reflected in `docs/FORMAT.md`, and must keep the
   cross-language reference vectors in both test suites passing. The two
   tools are interchangeable at the format boundary; drift is a bug.

## Getting set up

You need a Rust toolchain (1.70+) and Go (1.21+; the module targets 1.24).

```bash
make build     # rust/target/release/cairn and bin/cairn-run
make test      # cargo test + go test ./...
make demo      # build everything, then run examples/demo.sh
```

Run the full suite before opening a pull request:

```bash
