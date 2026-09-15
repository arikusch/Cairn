---
name: Bug report
about: Something in the engine or wrapper behaves incorrectly
title: ""
labels: bug
assignees: ""
---

**What happened**

A clear description of the bug.

**Command and output**

The exact command you ran and the full output (add `--verbose` for
`cairn-run`):

```console
$ cairn ...
```

**Expected behavior**

What you expected instead.

**Environment**

- `cairn version` / `cairn-run version`:
- OS and architecture:
- Toolchains (if you built from source): `rustc --version`, `go version`

**Cache state (if relevant)**

If the issue involves a specific key, paste the output of
`cairn show --key <key>` and the contents of your `.cairn-cache/` tree
(`find .cairn-cache -maxdepth 2`).

**Reproduction**

Minimal steps to reproduce; if a manifest or fixture is involved, attach it.
