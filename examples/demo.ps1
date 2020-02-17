# Cairn end-to-end demo (Windows PowerShell).
#
# Mirrors examples/demo.sh: build both tools, then show hashing, a cache MISS
# that runs a build, a cache HIT that skips it, and a cross-language check
# where the Rust engine restores/verifies the Go-written manifest.
$ErrorActionPreference = 'Stop'

$here  = Split-Path -Parent $PSScriptRoot
$cairn = Join-Path $here 'rust\target\release\cairn.exe'
$run   = Join-Path $here 'bin\cairn-run.exe'

