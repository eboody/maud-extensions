# maud_exts

Proc macros to simplify Maud views.

## Install
```toml
[dependencies]
maud-exts = "0.1.0"
```

## Macros
- `css! { ... }` or `css!("...")`: emits a `<style>` block with raw CSS.
- `js! { ... }` or `js!("...")`: emits a `<script>` block with raw JS.
- `inline_css! { ... }` and `inline_js! { ... }`: emit `fn css() -> maud::Markup` / `fn js() -> maud::Markup`.
- `font_face!` and `font_faces!`: inline font-face CSS as data URLs.

## Notes
- `css!` validates CSS using `cssparser`.
- `js!` validates JavaScript using `swc_ecma_parser`.
- `font_face!` uses `base64` at the call site, so the consuming crate must include `base64` if you use those macros.

## Example
```rust
use maud_exts::{css, js};

maud::html! {
    div {
        (css! {
            me { padding: 8px; }
        })
        button { "Ping" }
        (js! {
            me("-").on("click", () => { me("-").textContent = "Pong." })
        })
    }
}
```
