# Releasing `maud-extensions`

This file is the release checklist for the published crate surface.

## Before Tagging

1. Update `CHANGELOG.md`.
2. Re-read `README.md` against the current crate surface.
   Check these workflows explicitly:
   - `component!` with `js!` / `css!`
   - `ComponentBuilder`
   - Signals runtime injection
   - runtime slots via `maud-extensions-runtime`
3. Re-read the relevant rustdoc examples in `src/lib.rs`.
4. Confirm crate versions and `rust-version` fields.
5. Run:

```bash
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

6. Confirm any user-visible bug fix ships with a regression test.
7. Confirm compile-fail coverage still matches the intended proc-macro
   diagnostics when the release changes `component!` or `ComponentBuilder`.
8. Confirm vendored runtime assets are the intended versions when a release
   changes bundled JS.

## Publishing

1. Create the release commit.
2. Tag the release.
3. Publish `maud-extensions-runtime`.
4. Publish `maud-extensions`.
5. Push the commit and tag.

## After Publishing

1. Verify crates.io metadata and docs.rs builds.
2. Move the `Unreleased` entries in `CHANGELOG.md` into the released section.
3. Add any compatibility notes to the GitHub release if the release changes:
   - proc-macro diagnostics
   - bundled runtime behavior
   - `ComponentBuilder` builder rules
