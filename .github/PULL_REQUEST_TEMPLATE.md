## Summary

- what changed
- why it changed

## Release Gates

- [ ] `cargo fmt --check`
- [ ] `cargo test --all-targets`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --doc`
- [ ] `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`

## Public Surface

- [ ] docs updated for user-visible behavior changes
- [ ] regression tests added for bug fixes
- [ ] changelog updated when the change is release-visible
