# Pull request

## What and why

One or two sentences on the change and the problem it solves.

## Checklist

- [ ] One topic; commits use conventional subjects.
- [ ] `make test` passes locally (both toolchains).
- [ ] If behavior changed, a test that would have failed before is included.
- [ ] If the format is touched: both implementations updated,
      `docs/FORMAT.md` updated, reference vectors updated, format version
      bumped if incompatible.
- [ ] No new third-party dependencies (see CONTRIBUTING.md).
- [ ] `cargo fmt` and `gofmt` clean.

## Notes for reviewers

Anything that needs eyes: edge cases, trade-offs, or follow-ups.
