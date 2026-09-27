# Glossary

The vocabulary Cairn uses, in roughly the order the concepts appear in a run.

| Term | Meaning |
| --- | --- |
| content address | the canonical 16-character FNV-1a digest that names a blob |
| cache key | the digest of the declared inputs plus the command; names one logical build step |
| manifest | the canonical JSON record for a key: command, inputs, outputs |
| blob | an output file stored under its content address, sharded by the first two hex characters |
| hit | a manifest was found and its outputs were restored; the command did not run |
| miss | no manifest was found (or restore failed); the command ran and the outputs were captured |
| producer | which tool wrote a manifest: `cairn-rust` or `cairn-go` |
| format version | the integer that versions the key stream, the manifest, and the on-disk layout |
