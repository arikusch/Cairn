# Cairn Cache Format (v1)

This document is the normative specification of the Cairn on-disk format and
hashing scheme. Both the Rust engine (`cairn`) and the Go wrapper (`cairn-run`)
implement it exactly, so a cache produced by one tool is fully usable by the
other.

## 1. Content hashing

Cairn addresses content with the **64-bit FNV-1a** hash.

```
state = 0xcbf29ce484222325            (offset basis)
for each byte b in input:
    state = state XOR b
    state = (state * 0x00000100000001b3) mod 2^64   (wrapping multiply)
```

The **canonical digest** is the final `state` rendered as a zero-padded,
lowercase, 16-character hexadecimal string (`%016x`).

### Reference vectors

| Input      | Canonical digest     |
| ---------- | -------------------- |
