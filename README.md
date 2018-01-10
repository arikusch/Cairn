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
