# Reproducibility notes

A cache key sees only what you declare. Two invocations produce the same key
exactly when:

- the declared input bytes are identical,
- the command line is identical (every argument is folded into the key), and
- the format version matches.

Things the key deliberately does not see, and how to handle them:

| Not in the key | What to do |
| --- | --- |
| environment variables | declare files that capture them (lockfiles, toolchain pins) as inputs |
| toolchain version | record it in a declared input, or accept that a toolchain bump is invisible |
| the working directory | paths resolve against `--base-dir` and are normalized to forward slashes |
| undeclared files read by the command | declare every real input; the key cannot see what you did not name |

Undeclared state is the single largest source of stale hits; the failure mode
table in the README lists the rest.
