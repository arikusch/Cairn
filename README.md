<!-- Cairn — a field guide to a content-addressed build cache. -->

# Cairn — a field guide

<p align="center">
  <img src="docs/assets/strata.svg" alt="Build artifacts descending into content-addressed rock strata, each seam labelled with an FNV-1a digest shard" width="820">
</p>

> A cairn is a stack of stones left to mark that *someone has already been this
> way*. This Cairn does the same for a build: it marks work that has already
> been done, so you never have to walk it twice.

Cairn is a **universal, content-addressed build cache**. It hashes the inputs
to a build step, files the outputs in a deduplicated object store keyed by
content, and — when the same inputs come around again — hands the outputs back
instead of re-running the work. It does not care what language your build is
written in; it cares only about *bytes in* and *bytes out*.

The system is quarried from two tools that agree, seam for seam, on one format:

| Tool         | Language | Role in the section                                          |
| ------------ | -------- | ------------------------------------------------------------ |
| `cairn`      | Rust     | The engine — hashes, stores, restores, and *verifies* rock.  |
| `cairn-run`  | Go       | The surveyor — wraps any command; on a hit restores, not run.|

Both are built on standard libraries only — no third-party crates, no Go
modules beyond the toolchain. The content address is a **deterministic,
non-cryptographic FNV-1a** digest; that caveat is not a footnote — it is
load-bearing, and it is stated plainly throughout.

---

## Core sample (table of contents)

- [Surface reading — the problem](#surface-reading--the-problem)
- [Why two languages share one seam](#why-two-languages-share-one-seam)
- [Field kit — install &amp; build](#field-kit--install--build)
- [First descent — quickstart transcripts](#first-descent--quickstart-transcripts)
- [The three strata: MISS → STORE → HIT](#the-three-strata-miss--store--hit)
- [Reading the rock — data layout on disk](#reading-the-rock--data-layout-on-disk)
- [Cache-key anatomy](#cache-key-anatomy)
- [Rust ↔ Go interop](#rust--go-interop)
- [Guided walk — the scripted demo](#guided-walk--the-scripted-demo)
- [The performance ledger](#the-performance-ledger)
- [Field hazards — failure modes](#field-hazards--failure-modes)
- [Known limits of the survey](#known-limits-of-the-survey)
- [Integration recipes](#integration-recipes)
- [Command reference](#command-reference)
- [Roadmap — unexplored seams](#roadmap--unexplored-seams)
- [License](#license)

---

## Surface reading — the problem

Every build system re-does work it has already done: change one comment and the
toolchain recompiles a hundred files that did not move. Language-native caches
exist, but they are islands — Cargo does not know what `make` did, `go build`
does not know what your bundler did, and none share a cache with the shell
script gluing the pipeline together.

Cairn takes the opposite position. It treats a build step as an opaque
function — *declared inputs, a command, declared outputs* — and memoizes it by
content. If the inputs and the command hash to a key seen before, the recorded
outputs are laid back down verbatim and the command never runs. Because the key
is derived purely from bytes, a step cached by the Go wrapper is a legitimate
hit for the Rust engine, and the reverse — the common ground a polyglot pipeline
never otherwise had. It is deliberately a *small* idea executed carefully: Cairn
does not schedule, resolve dependency graphs, or discover inputs. You declare
them; it remembers them.

## Why two languages share one seam

A cache is only worth trusting if every tool that touches it computes the same
key and reads the same bytes. The two-language design exists to *prove* that
rather than assert it: the Go surveyor writes a manifest, the Rust engine
restores and verifies it, their keys match byte-for-byte, and the test suites
carry cross-language reference vectors so any drift fails CI immediately. The
contract that binds them is [`docs/FORMAT.md`](docs/FORMAT.md) — the normative
specification. This README is the field guide *to* that contract.

## Field kit — install &amp; build

You need a Rust toolchain (1.70+) and Go (1.21+; the module targets 1.24).

```bash
make build     # -> rust/target/release/cairn  and  bin/cairn-run
make test      # cargo test + go test ./...
make demo      # build everything, then run examples/demo.sh
make clean     # cargo clean + rm -rf bin .cairn-cache
```

To build each tool by hand: `cd rust && cargo build --release`, and
`cd go && go build -o ../bin/cairn-run ./cmd/cairn-run`.

## First descent — quickstart transcripts

Hash a file to see a content address (16 lowercase hex characters — a
zero-padded 64-bit FNV-1a digest), then wrap a real command. The first run is a
**cache MISS** (the command runs, its output is captured); the second is a
**cache HIT** (nothing executes, the output is laid back down):

```console
$ printf 'greetings from cairn' > input.txt
$ cairn hash input.txt
2e94eeb351265c82          20  input.txt

$ cairn-run --verbose --input input.txt --output output.txt -- \
      sh -c 'tr a-z A-Z < input.txt > output.txt'
cairn-run: cache MISS 8c165ae10d5fa0a2
cairn-run: stored manifest 8c165ae10d5fa0a2 (1 output(s))

$ rm output.txt   # then re-run the identical command
$ cairn-run --verbose --input input.txt --output output.txt -- \
      sh -c 'tr a-z A-Z < input.txt > output.txt'
cairn-run: cache HIT 8c165ae10d5fa0a2 (restored 1 output(s))
$ cat output.txt
GREETINGS FROM CAIRN
```

The output round-trips exactly; nothing recompiled. The [interop
section](#rust--go-interop) shows the engine reading and verifying this same
manifest.

## The three strata: MISS → STORE → HIT

Every wrapped command travels the same three-layer section:

```
  declared inputs ──FNV-1a──▶ input entries ─┐
  command line ──────────────────────────────┼─▶ CACHE KEY
                                              ┘
        │
        ▼   manifests/<key>.json exists?
   no ──┴── yes
   │         │
  MISS      HIT ── restore every output blob, skip the command
  run cmd
   │
  STORE ── capture outputs as content-addressed blobs, write manifest
```

Three rules govern the strata:

1. **MISS runs; HIT restores.** A hit never executes your command. If restoring
   a hit *fails* (say, a pruned blob), the surveyor does not error — it falls
   through and re-runs, then re-stores. The cache degrades to a miss, never to a
   broken build.
2. **STORE is content-addressed.** Outputs are hashed and filed under their own
   digest, so two steps that produce identical bytes share one blob on disk.
3. **Failure is never cached.** A non-zero command exit writes no manifest; a
   red build stays red on the next run.

## Reading the rock — data layout on disk

Relative to the cache root (default `.cairn-cache/`):

```
.cairn-cache/
├── objects/
│   ├── aa/aabb…            content blob — filename IS its digest
│   └── 85/8594…            sharded by the first two hex chars
└── manifests/
    └── <key>.json          one canonical manifest per cache key
```

- **Blobs are content-addressed and deduplicated.** A blob's path *is* its
  FNV-1a digest, sharded into a two-hex-character subdirectory. Identical output
  bytes are written exactly once, however many manifests reference them.
- **Writes are atomic.** Every blob and manifest is written to a `.tmp` sibling,
  `fsync`ed, and `rename`d into place, so a crash mid-write cannot leave a
  half-formed object where a reader might find it.
- **Manifests are the index bedrock.** A manifest names the command and the
  sorted input/output entries (`{path, digest, size}` each); restoring a key
  reads its manifest and copies the referenced blobs to their paths.

Manifests are written *canonically* — compact, fixed key order, RFC 8259 string
escaping — which is what lets two languages produce byte-identical files. Full
details live in [`docs/FORMAT.md`](docs/FORMAT.md).

## Cache-key anatomy

A cache key identifies one logical build step: the FNV-1a digest of a single
NUL-delimited byte stream, assembled in this exact order (`·` marks a `\x00`):

```
"cairn-key" ·
<format-version> ·                 (decimal ASCII, currently "1")
for each input entry, SORTED ascending by path:
    <path> · <digest> · <size> ·   (path uses forward slashes on every OS)
"cmd" ·
for each command argument, in order:
    <arg> ·
```

Three properties fall out of this layout: inputs are **sorted by path** before
hashing, so `-i b.c -i a.c` equals `-i a.c -i b.c`; **every command argument is
folded in**, so changing a flag or the command produces a different key and
unrelated steps never collide; and **paths are normalized to forward slashes**,
so a manifest written on Windows is a legitimate hit on Linux and vice-versa.
The format version is part of the stream, so bumping it cleanly invalidates
every old key rather than risking a silent misread.

### Reference vectors

Both test suites assert the canonical FNV-1a vectors — `""` →
`cbf29ce484222325`, `"a"` → `af63dc4c8601ec8c`, `"foobar"` →
`85944171f73967e8` — so the two implementations can never quietly diverge. The
full table lives in [`docs/FORMAT.md`](docs/FORMAT.md).

### The honest caveat — FNV-1a is not cryptographic

FNV-1a has **no collision resistance and no pre-image resistance** against a
deliberate adversary. Cairn is a build cache, not a security boundary. It was
chosen because it is deterministic across platforms and languages,
dependency-free (a few lines in any language), and fast enough to detect the
*accidental* change that is a cache's actual job.

Do **not** rely on a Cairn digest to detect malicious tampering. If you need
that, layer a signed manifest or a cryptographic digest *on top of* Cairn — do
