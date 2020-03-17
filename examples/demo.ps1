# Cairn end-to-end demo (Windows PowerShell).
#
# Mirrors examples/demo.sh: build both tools, then show hashing, a cache MISS
# that runs a build, a cache HIT that skips it, and a cross-language check
# where the Rust engine restores/verifies the Go-written manifest.
$ErrorActionPreference = 'Stop'

$here  = Split-Path -Parent $PSScriptRoot
$cairn = Join-Path $here 'rust\target\release\cairn.exe'
$run   = Join-Path $here 'bin\cairn-run.exe'

Write-Host '==> Building Rust engine'
Push-Location (Join-Path $here 'rust'); cargo build --release | Out-Null; Pop-Location
Write-Host '==> Building Go wrapper'
New-Item -ItemType Directory -Force -Path (Join-Path $here 'bin') | Out-Null
Push-Location (Join-Path $here 'go'); go build -o $run ./cmd/cairn-run; Pop-Location

$work = Join-Path $env:TEMP ("cairn-demo-" + (Get-Random))
New-Item -ItemType Directory -Path $work | Out-Null
Push-Location $work
try {
    Write-Host "`n==> Create a source input"
    Set-Content -Path input.txt -Value 'greetings from cairn' -NoNewline
    & $cairn hash input.txt

    Write-Host "`n==> First run (expect cache MISS, command executes)"
    & $run --verbose --input input.txt --output output.txt -- powershell -NoProfile -Command "(Get-Content input.txt).ToUpper() | Set-Content output.txt -NoNewline"
    Write-Host "output.txt: $(Get-Content output.txt)"

    Write-Host "`n==> Delete output, run again (expect cache HIT, command skipped)"
    Remove-Item output.txt
    & $run --verbose --input input.txt --output output.txt -- powershell -NoProfile -Command "(Get-Content input.txt).ToUpper() | Set-Content output.txt -NoNewline"
    Write-Host "output.txt restored: $(Get-Content output.txt)"

    Write-Host "`n==> Cross-language: Rust computes the same key"
    $key = & $cairn key --input input.txt -- powershell -NoProfile -Command "(Get-Content input.txt).ToUpper() | Set-Content output.txt -NoNewline"
    Write-Host "key = $key"

    Write-Host "`n==> Rust restores + verifies the Go-written manifest"
    Remove-Item output.txt
