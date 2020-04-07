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
| `""`       | `cbf29ce484222325`   |
| `"a"`      | `af63dc4c8601ec8c`   |
| `"foobar"` | `85944171f73967e8`   |

Both implementations assert these vectors in their unit tests.

### ⚠️ Non-cryptographic

FNV-1a is **not** a cryptographic hash. It has no collision resistance or
pre-image resistance against a deliberate attacker. Cairn is a build cache, not
a security boundary. FNV-1a was chosen because it is:

- **Deterministic** across platforms, architectures, and languages.
- **Dependency-free** — expressible in a few lines in any language.
- **Fast** and adequate for detecting *accidental* change (the actual job of a
  build cache).

Do not use Cairn digests to defend against malicious tampering. If you need
that, layer a signed manifest or a cryptographic digest on top.

## 2. Cache key

A cache key identifies a logical build step. It is the FNV-1a canonical digest
of the following byte stream, where `\x00` is a single NUL byte:

```
"cairn-key\x00"
<format-version-decimal> "\x00"
for each input entry, sorted ascending by path:
    <path>   "\x00"
    <digest> "\x00"
    <size-decimal> "\x00"
"cmd\x00"
for each command argument, in order:
    <arg> "\x00"
```

- `format-version-decimal` is the decimal ASCII of the format version (`1`).
- Inputs **must** be sorted by `path` (byte-wise ascending) before hashing.
- `path` uses forward slashes on all platforms.
- `size-decimal` is the decimal ASCII of the byte length.

Because the command participates in the key, changing the command (or any
argument) yields a different key, so unrelated steps never collide.

## 3. Manifest JSON
