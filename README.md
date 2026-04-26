# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)

Proc macros for Maud with a deliberately small default story.

## Install

```bash
cargo add maud-extensions
```

If you want the crate to read as `mx::...` at call sites without renaming the
published crate:

```bash
cargo add maud-extensions --rename mx
```

or in `Cargo.toml`:

```toml
[dependencies]
mx = { package = "maud-extensions", version = "0.5.4" }
```

## Beautiful Default

Write plain `html!` and emit local CSS and JS where they belong:

```rust
use maud::html;
use maud_extensions::{css, js};

fn status_card(message: &str) -> maud::Markup {
    html! {
        article class="status-card" {
            h2 { "System status" }
            p class="message" { (message) }

            (css! {
                me {
                    border: 1px solid #ddd;
                    border-radius: 10px;
                    padding: 12px;
                }
                me.ready {
                    border-color: #16a34a;
                }
            })

            (js!(once, {
                me().class_add("ready");
            }))
        }
    }
}
```

This is the intended center of gravity:

- no wrapper component macro
- no hidden CSS/JS injection
- no stringly helper names
- plain Maud remains the main language

## Named Helpers

When reuse helps, define local helper functions with Rust identifiers:

```rust
use maud::html;
use maud_extensions::{css, js};

css!(card_css, {
    me { gap: px!(12); }
});

js!(card_js, once, {
    me().class_add("ready");
});

fn card() -> maud::Markup {
    html! {
        article.card {
            (card_css())
            (card_js())
            "Hello"
        }
    }
}
```

Supported `css!` forms:

- `css! { ... }`
- `css!(name, { ... })`

Supported `js!` forms:

- `js! { ... }`
- `js!(once, { ... })`
- `js!(name, { ... })`
- `js!(name, once, { ... })`

## CSS Helper Macros

Inside `css!` token mode you can use:

- `raw!(r#"..."#)`
- `media!(prelude, { ... })`
- `container!(prelude, { ... })`
- `supports!(prelude, { ... })`
- `layer!(prelude, { ... })`
- `keyframes!(prelude, { ... })`
- unit helpers:
  - `rem!(...)`
  - `em!(...)`
  - `px!(...)`
  - `pct!(...)`
  - `vw!(...)`
  - `vh!(...)`
  - `ms!(...)`
  - `s!(...)`

Example:

```rust
use maud_extensions::css;

fn responsive_styles() -> maud::Markup {
    css! {
        media!("(min-width: 48rem)", {
            me { padding: rem!(2); }
        })
        supports!("(display: grid)", {
            me { gap: px!(12); }
        })
    }
}
```

## Limits

- `css!` and `js!` are placement-sensitive local emitters
- `js!(once, ...)` relies on a `data-mx-js-ran` marker on the parent element
- CSS token mode only sees Rust-tokenizable input; use `raw!(...)` for arbitrary
  CSS fragments
- JavaScript is validated with SWC before emission
- CSS is checked for lightweight syntax and raw-text safety before emission
