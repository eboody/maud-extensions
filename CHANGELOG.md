# Changelog

All notable changes to this project should be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.5.0] - 2026-04-15

### Added

- Added `#[derive(ComponentBuilder)]` for typed shell and layout components.
- Added generated builder support for:
  - required fields
  - optional `Option<T>` fields
  - repeated `Vec<T>` fields
  - `#[builder(default)]`
  - `#[builder(each = "...")]`
  - `#[slot]` and `#[slot(default)]` metadata
- Added compile-tested coverage for `ComponentBuilder` success and failure
  cases.
- Added bundled Signals runtime helpers:
  `signals_inline!()` and `surreal_scope_signals_inline!()`.
- Added the JS-first Signals binder surface on `window.mx` and the
  `me(...).bind*` convenience path.

### Changed

- Reworked the README as a landing page around the actual crate workflows:
  `component!`, `ComponentBuilder`, Signals, runtime injection, and slots.
- Reframed runtime slots as the lower-level transport layer and moved
  `ComponentBuilder` into the default path for new shell/layout components.

### Fixed

- Hardened `ComponentBuilder` against raw-identifier and duplicate generated
  method-name edge cases.
- Hardened `ComponentBuilder` field-kind classification so redundant
  `#[builder(default)]` on `Option<T>` and `Vec<T>` fields doesn't break the
  generated ergonomic setters.
- Narrowed `ComponentBuilder` markup/type special handling to the lexical type
  forms the derive can actually observe.
