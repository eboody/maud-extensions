# Releasing `maud-extensions`

## Before tagging

1. Update `CHANGELOG.md`.
2. Confirm the crate versions and `rust-version` fields.
3. Run:

```bash
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

4. Re-read the README and rustdoc examples for drift.
5. Confirm any user-visible bug fix ships with a regression test.

## Publishing

1. Create the release commit.
2. Tag the release.
3. Publish `maud-extensions-runtime`.
4. Publish `maud-extensions`.
5. Push the commit and tag.

## After publishing

1. Verify crates.io metadata and docs.rs builds.
2. Move the `Unreleased` entries in `CHANGELOG.md` into the released section.
3. Note any compatibility or migration guidance in the GitHub release notes.
