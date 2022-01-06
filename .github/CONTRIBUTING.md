# Contributing to Cairn

Thanks for your interest in Cairn. This document describes how to propose
changes that fit the project's constraints.

## Ground rules

Cairn is deliberately small and boring. Two rules decide whether a change
can be merged:

1. **Standard libraries only.** The Rust engine uses no third-party crates
   and the Go wrapper uses no modules beyond the toolchain. A pull request
   that adds a dependency needs a very strong argument and will usually be
