# On-disk layout

```
cache/
  manifests/   <digest>.json   - input digests -> output blob list
  blobs/       <digest>        - stored output bytes, content-addressed
```

The manifest is the unit of restore; blobs are shared across manifests,
which is what makes repeated builds cheap. Digests are FNV-1a - verify on
restore byte-compares, so a collision degrades to a miss, not a bad hit.
