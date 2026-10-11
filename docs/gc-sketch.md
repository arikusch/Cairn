# Incremental GC sketch

Manifests already reference every blob they need. A mark pass over the
manifest set, followed by a sweep of the object store, reclaims unreferenced
blobs without touching the content address format.

Open question: whether the sweep runs inline after a store or as an explicit
`cairn gc` step. Both keep the byte-guarantees intact.
