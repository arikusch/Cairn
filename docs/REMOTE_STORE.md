# Sketch: a remote store (not part of the format)

This document is a design sketch, not a specification. Nothing here ships in
the current release.

- **Transport**: the existing paths are already content-addressed, so a dumb
  HTTP transport can serve `objects/<aa>/<digest>` and `manifests/<key>.json`
  with no new encoding.
- **Push/pull**: a CI job pushes manifests and blobs; a developer pulls by key.
- **Trust**: FNV-1a is not cryptographic (`docs/FORMAT.md`), so a remote store
  must be treated as untrusted. Verification by re-hashing on fetch is
  mandatory, and a signed manifest belongs on top of the existing digest,
  never instead of it.
- **Eviction**: a remote store needs the same GC story as the local one
  before it can be operated honestly.
