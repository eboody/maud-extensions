# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)

Small, local superpowers for Maud.

## Install

Recommended: install the crate as `mx` so the macro and component surface reads
well at call sites:

```bash
cargo add maud-extensions --rename mx
```

If you want the experimental component system too, enable `components` on the
same dependency:

```toml
[dependencies]
mx = { package = "maud-extensions", version = "0.6.3", features = ["components"] }
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

The macro that turns the impl-local component surface on is:

```rust
#[mx::component]
impl Card {
    render! { ... }
    css! { ... }
    js!(once, { ... });
}
```

That impl macro is what makes `render!`, `css!`, and `js!` part of the
component render pipeline.

Preferred authoring pattern:

```rust
use maud::Markup;
use mx::{Component, Slot};

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
- no manual `impl Render` glue just to stitch component-local CSS/JS into the
  output

Current constraints:

- use `Slot<T>` / `Slot<Vec<T>>` for slot declarations
- if there are multiple slot fields, mark exactly one `#[mx(default)]`
- repeated slots use `Slot<Vec<T>>` plus `#[mx(each = item_name)]`
- `#[mx::component]` impls currently allow:
  - exactly one `render!`
  - at most one `css!`
  - at most one `js!`

Mental model:

- `#[derive(Component)]` owns fields, slots, and the Bon-backed builder
- `#[mx::component]` owns the render root and colocated CSS/JS blocks
- builder `.render()` goes through the render hook produced by the impl macro

This keeps the builder and the impl-local render/assets story explicit without
requiring manual `(Self::css())` / `(Self::js())` emission in the render body.

## Bundled browser-side building blocks

The current CSS/JS/component story is designed to layer on top of a few small
browser-side tools:

- [Surreal](https://github.com/gnat/surreal) for DOM ergonomics around `me()` /
  `any()` style behavior
- [css-scope-inline](https://github.com/gnat/css-scope-inline) for colocated
  scoped CSS transforms
- [Preact Signals](https://github.com/preactjs/signals) for the signals runtime
  surface the crate builds on

`maud-extensions` is not trying to replace those pieces; it is trying to make
them feel coherent and component-local from Maud.

To bootstrap the browser-side runtime in a page, emit one of the bundled
runtime macros in `<head>`:

```rust
use maud::html;
use mx::surreal_scope_signals_inline;

fn page() -> maud::Markup {
    html! {
        head {
            (surreal_scope_signals_inline!())
        }
        body {
            // component markup here
        }
    }
}
```

Available bundles:

- `surreal_scope_inline!()`
- `signals_inline!()`
- `surreal_scope_signals_inline!()`

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
