# maud_extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)
[![license](https://img.shields.io/crates/l/maud-extensions.svg)](https://github.com/eboody/maud-extensions)

Proc macros to simplify Maud views with inline CSS, JS, and font helpers.

## Install
```bash
cargo add maud-extensions
```

## Quick start
```rust
use maud_extensions::{css, js};

maud::html! {
    div class="card" {
        (css! {
            me { padding: 8px; border: 1px solid #ddd; }
            me em { color: #c00; }
        })
        p { "Inline CSS and JS" }
        (js! {
            me().class_add("ready");
        })
    }
}
```

## Macro overview
- `css! { ... }` or `css!("...")`
  - Emits a `<style>` block with raw CSS.
  - Validates CSS using `cssparser`.
- `js! { ... }` or `js!("...")`
  - Emits a `<script>` block with raw JS.
  - Validates JavaScript using `swc_ecma_parser`.
- `inline_css! { ... }` and `inline_js! { ... }`
  - Generate `fn css() -> maud::Markup` / `fn js() -> maud::Markup` helpers.
- `font_face!` and `font_faces!`
  - Inline font-face CSS as data URLs.

## CSS scoping pattern
These macros are often paired with a tiny JS scoper like
[`css-scope-inline`](https://github.com/gnat/css-scope-inline). It rewrites
selectors like `me { ... }` to a unique class on the parent element.

Example:
```rust
(css! {
    me { border: 1px dashed var(--accent); }
    me em { font-style: normal; }
})
```

## Inline JS helpers
The JS macro is format-preserving but may add spacing for token safety. It
still emits valid JavaScript. The examples use
[`surreal`](https://github.com/gnat/surreal) for the `me()` helper, but any
inline script works.

```rust
(js! {
    me().class_add("pinged");
})
```

## Font helpers
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
