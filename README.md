# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)

Small, local superpowers for Maud.

## Install

```bash
cargo add maud-extensions
```

If you want the experimental component system too:

```bash
cargo add maud-extensions --features components
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

## Core Story

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

This is still the intended center of gravity:

- no wrapper component macro
- no hidden CSS/JS injection
- no stringly helper names
- plain Maud remains the main language

## Experimental Components

The component system is opt-in behind the `components` feature.

Preferred authoring pattern:

```rust
use maud::{Markup, Render};
use maud_extensions::{self as mx, Component, Slot};

#[derive(Component)]
struct Card {
    title: String,
    header: Slot<Markup>,
    #[mx(default)]
    body: Slot<Markup>,
    footer: Slot<Markup>,
    #[mx(each = action)]
    actions: Slot<Vec<Markup>>,
}

#[mx::component]
impl Card {
    css! {
        me {
            padding: 1rem;
            border: 1px solid #ddd;
        }

        me .actions {
            display: flex;
            gap: 0.5rem;
        }
    }

    js!(once, {
        me().class_add("ready");
    });

    render! {
        article.card {
            header class="header" { (self.header) }
            h2 { (self.title) }
            div.body { (self.body) }
            footer class="footer" { (self.footer) }
            div.actions { (self.actions) }
        }
    }
}

impl Render for Card {
    fn render(&self) -> Markup {
        mx::ComponentRender::__mx_render(self)
    }
}

fn view() -> Markup {
    Card::new()
        .title("Profile")
        .header(maud::html! { span { "Welcome" } })
        .child(maud::html! { p { "Body" } })
        .footer(maud::html! { button { "Save" } })
        .action(maud::html! { button { "Edit" } })
        .render()
}
```

What this gives you:

- Bon-backed typed builders
- `Slot<Markup>` and `Slot<Vec<Markup>>` as the slot declaration path
- `#[mx(default)]` for the single default slot
- `#[mx::component] impl` blocks with colocated:
  - `render! { ... }`
  - `css! { ... }`
  - `js!(once, { ... })`
- builder `.render()` automatically includes impl-local CSS and JS in the
  rendered root

Current constraints:

- use `Slot<T>` instead of `#[mx(slot)]`
- if there are multiple slot fields, mark exactly one `#[mx(default)]`
- repeated slots use `Slot<Vec<T>>` plus `#[mx(each = item_name)]`
- `#[mx::component]` impls currently allow:
  - exactly one `render!`
  - at most one `css!`
  - at most one `js!`

Mental model:

- `#[derive(Component)]` owns fields, slots, and the Bon-backed builder
- `#[mx::component]` owns the render root and colocated CSS/JS blocks
- builder `.render()` goes through the hidden `ComponentRender` hook produced
  by the impl macro

This keeps the builder and the impl-local render/assets story explicit without
requiring manual `(Self::css())` / `(Self::js())` emission in the render body.

If you want a more minimal component style, you can still stop at plain
`html!` + `css!` + `js!`. The component system is intentionally a second layer,
not the only way to use the crate.

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
