# Changelog

All notable changes to this project should be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

- Hardened proc-macro and slot-runtime internals.
- Added public-surface docs, examples, and broader test coverage.
- Added CI, issue templates, and release gates.
- Added bundled Signals runtime helpers: `signals_inline!()` and
  `surreal_scope_signals_inline!()`.
- Added the JS-first Signals binder surface on `window.mx` and the
  `me(...).bind*` convenience path.
