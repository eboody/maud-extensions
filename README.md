# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)
[![license](https://img.shields.io/crates/l/maud-extensions.svg)](https://github.com/eboody/maud-extensions)

Proc macros for Maud that make inline CSS/JS and component-style authoring simpler.

This crate includes bundled copies of
[gnat/surreal](https://github.com/gnat/surreal) and
[gnat/css-scope-inline](https://github.com/gnat/css-scope-inline). Check those
repos to see what these two tiny JS files can do and how to use them.

## Why use it?
- Define component-local `js()` and `css()` helpers with short macros.
- Wrap markup with `component!` and auto-inject `(js())` + `(css())`.
- Emit direct `<script>` / `<style>` blocks when needed.
- Bundle `surreal.js` and `css-scope-inline.js` with zero path setup.
- Embed fonts as base64 `@font-face` CSS.

## Table of Contents
- [Install](#install)
- [Quick Start](#quick-start)
- [component!](#component)
- [Runtime Injection](#runtime-injection)
- [Macro Reference](#macro-reference)
- [Migration (Breaking)](#migration-breaking)
- [Font Helpers](#font-helpers)
- [License](#license)

## Install

```bash
cargo add maud-extensions
```

## Quick Start

```rust
use maud::{html, Markup, Render};
use maud_extensions::{component, css, js, surreal_scope_inline};

struct StatusCard<'a> {
    message: &'a str,
}

impl<'a> Render for StatusCard<'a> {
    fn render(&self) -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            article class="status-card" {
                h2 { "System status" }
                p class="message" { (self.message) }
            }
        };

        css! {
            me {
                border: 1px solid #ddd;
                border-radius: 10px;
                padding: 12px;
                transition: border-color 160ms ease-in;
            }
            me.ready {
                border-color: #16a34a;
            }
            me .message {
                margin: 0;
                opacity: 0.85;
            }
        }

        view
    }
}

struct Page;

impl Render for Page {
    fn render(&self) -> Markup {
        html! {
            head {
                // Inject bundled `surreal.js` + `css-scope-inline.js`.
                (surreal_scope_inline!())
            }
            body {
                (StatusCard { message: "All systems operational" })
            }
        }
    }
}
```

## `component!`

`component!` wraps one top-level Maud element and appends `(js())` and `(css())`
inside that root element automatically.

```rust
use maud::{Markup, Render};
use maud_extensions::{component, css, js};

struct Card;

impl Render for Card {
    fn render(&self) -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            section class="card" {
                p { "Hello" }
            }
        };

        css! {
            me { border: 1px solid #ddd; }
        }

        view
    }
}
```

Equivalent output shape:
- root element content
- then `(js())`
- then `(css())`

Rules:
- input must be exactly one top-level element with a `{ ... }` body
- `js()` and `css()` helpers must already be in scope
- trailing `;` is allowed
- invalid root shapes fail at compile time with guidance

## Runtime Injection

Use bundled runtime scripts with no filesystem setup:

```rust
use maud_extensions::surreal_scope_inline;

maud::html! {
    (surreal_scope_inline!())
}
```

Need custom files instead? Use `js_file!` / `css_file!` (`include_str!` behavior):

```rust
use maud_extensions::js_file;

maud::html! {
    (js_file!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/vendor/custom-runtime.js")))
}
```

## Macro Reference

- `js! { ... }` / `js!("...")`
  - Generate local `fn js() -> maud::Markup` helper.
- `css! { ... }` / `css!("...")`
  - Generate local `fn css() -> maud::Markup` helper.
- `component! { ... }`
  - Wrap one root element and inject `(js())` + `(css())` at the end of its body.
- `inline_js! { ... }` / `inline_js!("...")`
  - Emit `<script>` markup directly.
  - Validate JS via `swc_ecma_parser`.
- `inline_css! { ... }` / `inline_css!("...")`
  - Emit `<style>` markup directly.
  - Validate CSS via `cssparser`.
- `js_file!("path")` / `css_file!("path")`
  - Emit `<script>` / `<style>` tags from file contents.
- `surreal_scope_inline!()`
  - Emit bundled `surreal.js` and `css-scope-inline.js`.
- `font_face!(...)` / `font_faces!(...)`
  - Embed font files as base64 `@font-face` CSS.

## Migration (Breaking)

The JS/CSS macro names were intentionally swapped.

- old `js!` -> now `inline_js!`
- old `css!` -> now `inline_css!`
- old `inline_js!` -> now `js!`
- old `inline_css!` -> now `css!`

If you were manually placing `(js())` / `(css())` inside root markup, you can now
switch to `component!` and remove those explicit calls.

## Font Helpers

`font_face!` and `font_faces!` embed font files as base64 data URLs. Because
this macro expands at the call site, the consuming crate must include `base64`
if you use these macros.

```rust
use maud_extensions::font_face;

maud::html! {
    (font_face!(
        "../static/fonts/JetBrainsMono.woff2",
        "JetBrains Mono"
    ))
}
```

## License

MIT OR Apache-2.0
