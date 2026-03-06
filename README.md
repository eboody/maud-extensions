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
- Keep CSS and JS close to the Maud view where they are used.
- Validate inline CSS (`css!`) and JS (`js!`) at compile time.
- Use an SFC-like flow with `inline_js!` + markup + `inline_css!`.
- Inline runtime files like `surreal.js` and `css-scope-inline.js` with one macro.
- Embed fonts as base64 `@font-face` blocks.

## Table of Contents
- [Install](#install)
- [Quick Start](#quick-start)
- [SFC-Style Component Flow](#sfc-style-component-flow)
- [Inject JS/CSS Files](#inject-jscss-files)
- [Macro Reference](#macro-reference)
- [CSS Scoping Pattern](#css-scoping-pattern)
- [Font Helpers](#font-helpers)
- [License](#license)

## Install

```bash
cargo add maud-extensions
```

## Quick Start

```rust
use maud_extensions::{inline_css, inline_js, surreal_scope_inline};

fn status_card(message: &str) -> maud::Markup {
    inline_js! {
        me().class_add("ready");
    }

    let view = maud::html! {
        article class="status-card" {
            h2 { "System status" }
            p class="message" { (message) }
            (js())
            (css())
        }
    };

    inline_css! {
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

fn page() -> maud::Markup {
    maud::html! {
        head {
            // Inject bundled `surreal.js` + `css-scope-inline.js`.
            (surreal_scope_inline!())
        }
        body {
            (status_card("All systems operational"))
        }
    }
}
```

## SFC-Style Component Flow

You can structure a Maud component like a single-file component:
- script at the top
- template/markup in the middle
- style at the bottom

```rust
use maud_extensions::{inline_css, inline_js};

fn profile_card() -> maud::Markup {
    inline_js! {
        me().class_add("hydrated");
    }

    let view = maud::html! {
        article class="profile-card" {
            h2 { "Maud component" }
            p { "Script on top, markup in the middle, style at the bottom." }
            (js())
            (css())
        }
    };

    inline_css! {
        me {
            border: 1px solid #ddd;
            border-radius: 10px;
            padding: 12px;
        }
    }

    view
}
```

`inline_js!` and `inline_css!` generate local helpers:
- `fn js() -> maud::Markup`
- `fn css() -> maud::Markup`

## Inject JS/CSS Files

Use the bundled runtime macro when you want zero path setup:

```rust
use maud_extensions::surreal_scope_inline;

maud::html! {
    (surreal_scope_inline!())
}
```

### Inject a single file

Use file-include macros for your own custom files via Rust's `include_str!`
behavior:

```rust
use maud_extensions::js_file;

maud::html! {
    (js_file!(concat!(env!("HOME"), "/code/eran_codes/crates/http/static/surreal.js")))
}
```

`surreal_scope_inline!()` emits two `<script>` tags:
- bundled `surreal.js`
- bundled `css-scope-inline.js`

## Macro Reference

- `css! { ... }` / `css!("...")`
  - Emit a `<style>` block.
  - Validate CSS via `cssparser`.
- `js! { ... }` / `js!("...")`
  - Emit a `<script>` block.
  - Validate JS via `swc_ecma_parser`.
- `inline_css! { ... }` / `inline_js! { ... }`
  - Generate local `css()` / `js()` helpers for SFC-style layout.
- `css_file!("path")` / `js_file!("path")`
  - Emit `<style>` / `<script>` tags from file contents.
- `surreal_scope_inline!()`
  - Emit bundled `surreal.js` and `css-scope-inline.js` without path setup.
- `font_face!(...)` / `font_faces!(...)`
  - Embed font files as base64 `@font-face` CSS.

## CSS Scoping Pattern

These macros pair well with
[`css-scope-inline`](https://github.com/gnat/css-scope-inline), which rewrites
selectors like `me { ... }` to a generated class on the current element.

```rust
(css! {
    me { border: 1px dashed var(--accent); }
    me em { font-style: normal; }
})
```

The examples use [`surreal`](https://github.com/gnat/surreal) for the `me()`
helper, but any inline JS can be used.

## Font Helpers

`font_face!` and `font_faces!` embed font files as base64 data URLs. Because
this macro expands at the call site, the consuming crate must include `base64`
if you use these macros.

```rust
use maud_extensions::font_face;

maud::html! {
    (font_face!("static/fonts/JetBrainsMono.woff2", "JetBrains Mono"))
}
```

## License

MIT OR Apache-2.0
