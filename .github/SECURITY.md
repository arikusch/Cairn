# Security Policy

## Supported versions

Cairn follows semantic versioning. Security fixes are issued for the latest
release line only.

| Version | Supported          |
| ------- | ------------------ |
| 5.1.x   | :white_check_mark: |
| 4.x     | :x:                |
| 3.x     | :x:                |
| 2.x     | :x:                |
| < 1.0   | :x:                |

## Reporting a vulnerability

Please **do not** open a public issue for a security concern. Use GitHub's
private vulnerability reporting:

1. Go to the repository's **Security** tab.
2. Click **Report a vulnerability**.
3. Describe the issue, the affected version, and a reproduction if you have
   one.

You can expect an acknowledgement within a few days. If the report is
accepted, a fix and a coordinated disclosure will follow; credit is given in
the release notes unless you prefer otherwise.

## Scope note: the hash is not a security boundary

Cairn addresses content with the 64-bit FNV-1a hash. It has **no collision
resistance and no pre-image resistance** against a deliberate adversary, and
that is by design: Cairn is a build cache, not a verification system. Digest
collisions are in scope as robustness bugs, but you should not rely on Cairn
to detect malicious tampering — layer a signed manifest or a cryptographic
digest on top if you need that. See `docs/FORMAT.md` for the full rationale.

Reports that amount to "FNV-1a is not cryptographic" will be closed as by
design. Reports about a **crash, path escape, or corruption** reachable
through a crafted manifest or cache directory are very much in scope.

<!-- draft note 1054 -->
