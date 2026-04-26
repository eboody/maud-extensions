# Changelog

All notable changes to this project should be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.6.1] - 2026-04-26

### Fixed

- Corrected the declared MSRV to Rust 1.88 to match the current dependency
  graph and updated CI accordingly.

## [0.6.0] - 2026-04-26

### Added

- Added the experimental component system behind the `components` feature with:
  - `#[derive(Component)]` Bon-backed builders
  - `Slot<Markup>` / `Slot<Vec<Markup>>` slot declarations
  - `#[mx::component]` impl blocks with `render!`, `css!`, and `js!`
  - builder `.render()` auto-injecting impl-local CSS and JS into the rendered root
- Added compile-fail coverage for unsupported component declarations and
  malformed `#[mx::component]` impl blocks.
- Split the proc-macro implementation into `maud-extensions-macros` and
  re-exported `bon` from the runtime crate so downstream users only need
  `maud-extensions`.

### Changed

- Refocused the crate around a small core (`html!` + local `css!` / `js!`) plus
  an opt-in experimental component layer.
- Refactored CSS and JS macro internals into semantic modules with shared
  diagnostics and clearer compile-time messages.

### Removed

- Removed the older component-centric surfaces from the public path in favor of
  the new experimental component model.

## [0.5.1] - 2026-04-17

### Added

- Added consuming `.render()` on complete `ComponentBuilder` builders.
- Added `maybe_<field>(Option<T>)` helpers for optional `ComponentBuilder`
  fields.
- Added named CSS helper support via `css! { "name", { ... } }`.

### Changed

- Updated README and rustdoc examples to match the generated
  `ComponentBuilder` surface.

### Fixed

- Added compile-fail coverage for `ComponentBuilder` `.render()` and optional
  `maybe_` helper name collisions.
- Added compile-fail coverage for invalid named `css!` helper identifiers.

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
