#!/usr/bin/env bash
#
# Cairn end-to-end demo.
#
# Builds both tools, then walks through: hashing, a cache MISS that runs a
# "build" and caches its output, a cache HIT that skips the build, and a
# cross-language check where the Rust engine restores and verifies the
# manifest the Go wrapper wrote.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cairn="$here/rust/target/release/cairn"
run="$here/bin/cairn-run"
