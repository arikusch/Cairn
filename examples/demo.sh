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

echo "==> Building Rust engine"
(cd "$here/rust" && cargo build --release >/dev/null)
echo "==> Building Go wrapper"
mkdir -p "$here/bin"
(cd "$here/go" && go build -o "$run" ./cmd/cairn-run)

# Ext for Windows.
if [[ "${OS:-}" == "Windows_NT" ]]; then
  cairn="$cairn.exe"; run="$run.exe"
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

echo
echo "==> Create a source input"
printf 'greetings from cairn' > input.txt
"$cairn" hash input.txt

echo
echo "==> First run (expect cache MISS, command executes)"
"$run" --verbose --input input.txt --output output.txt -- \
  sh -c 'tr a-z A-Z < input.txt > output.txt'
echo "output.txt: $(cat output.txt)"

echo
echo "==> Delete output, run again (expect cache HIT, command skipped)"
rm -f output.txt
"$run" --verbose --input input.txt --output output.txt -- \
  sh -c 'tr a-z A-Z < input.txt > output.txt'
echo "output.txt restored: $(cat output.txt)"

echo
echo "==> Cross-language: Rust computes the same key"
