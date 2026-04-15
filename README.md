# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)
[![license](https://img.shields.io/crates/l/maud-extensions.svg)](https://github.com/eboody/maud-extensions)

Proc macros for Maud that make component-style view code, bundled browser
runtime helpers, and typed shell components easier to author.

This crate has three main jobs:
- file-scoped component helpers with `js!`, `css!`, and `component!`
- direct emitters and bundled runtimes such as `inline_js!()`,
  `surreal_scope_inline!()`, and `surreal_scope_signals_inline!()`
- typed builder generation for layout and shell components with
  `#[derive(ComponentBuilder)]`

Signals support stays JS-first: Maud markup provides anchors, while `js!`
owns signals, effects, and DOM binding. The companion crate
[`maud-extensions-runtime`](https://crates.io/crates/maud-extensions-runtime)
still provides the older string-based slot transport, but for new shell and
layout components the preferred path is `ComponentBuilder`.

## Install

```bash
cargo add maud-extensions
cargo add maud-extensions-runtime # only needed for the lower-level runtime slot transport
```

Support policy:
- MSRV: Rust 1.85
- supported Maud version: 0.27

## Choose A Workflow

Use `component!` when the component owns its own DOM, local CSS, and local JS:

```rust
use maud::{html, Markup, Render};
use maud_extensions::{component, css, js, surreal_scope_inline};

js! {
    me().class_add("ready");
}

struct StatusCard<'a> {
    message: &'a str,
}

impl<'a> Render for StatusCard<'a> {
    fn render(&self) -> Markup {
        component! {
            @js-once
            article class="status-card" {
                h2 { "System status" }
                p class="message" { (self.message) }
            }
        }
    }
}

css! {
    me {
        border: 1px solid #ddd;
        border-radius: 10px;
        padding: 12px;
    }
    me.ready {
        border-color: #16a34a;
    }
}

let page = html! {
    head { (surreal_scope_inline!()) }
    body { (StatusCard { message: "All systems operational" }) }
};
```

Use `#[derive(ComponentBuilder)]` for new shell and layout components with
props and named content regions. This is the preferred path when the regions
can be expressed as typed fields:

```rust
use maud::{html, Markup, Render};
use maud_extensions::ComponentBuilder;

#[derive(Clone)]
struct ActionButton {
    label: &'static str,
}

impl Render for ActionButton {
    fn render(&self) -> Markup {
        html! { button type="button" { (self.label) } }
    }
}

#[derive(ComponentBuilder)]
struct Card {
    tone: &'static str,
    #[slot]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
    #[builder(each = "action")]
    actions: Vec<ActionButton>,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class={ "card " (self.tone) } {
                @if let Some(header) = &self.header {
                    header class="card-header" { (header) }
                }
                main class="card-body" { (self.body) }
                @if !self.actions.is_empty() {
                    footer class="card-actions" {
                        @for action in &self.actions {
                            (action)
                        }
                    }
                }
            }
        }
    }
}

let view = Card::new()
    .tone("info")
    .header(html! { h2 { "Status" } })
    .body(html! { p { "All systems green" } })
    .action(ActionButton { label: "Retry" })
    .action(ActionButton { label: "Dismiss" })
    .build()
    .render();
```

Use the runtime slot API from `maud-extensions-runtime` only when you really
need open caller-owned child structure, or when you are keeping an existing
slot-based component. This is the lower-level string-based transport layer:

```rust
use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

struct Panel;

impl Render for Panel {
    fn render(&self) -> Markup {
        html! {
            article {
                header { (named_slot("header")) }
                main { (slot()) }
            }
        }
    }
}

let view = html! {
    (Panel.with_children(html! {
        (html! { h2 { "Status" } }.in_slot("header"))
        p { "Body content" }
    }))
};
```

## `ComponentBuilder`

`ComponentBuilder` is the preferred API for new typed shell and layout
components. It doesn't try to invent new Maud syntax. It generates a normal
Rust builder from the component struct you already want to render.

What it generates:
- `Type::new()` and `Type::builder()`
- one setter per field
- `#[builder(each = "...")]` item setters for `Vec<T>` fields
- `.build()` once all required fields are present
- `From<CompleteBuilder> for Type`

Field rules:
- plain fields are required
- `Option<T>` fields are optional
- `Vec<T>` fields default to empty and can use `#[builder(each = "...")]`
- `#[builder(default)]` makes a non-`Option`, non-`Vec` field use `Default`
- `#[slot]` and `#[slot(default)]` record the component's content-region
  contract for this builder-core layer and for later syntax sugar

Markup ergonomics:
- fields written as `Markup`, `maud::Markup`, or `::maud::Markup` accept any
  `impl Render` in generated setters
- that applies to single-markup fields, optional markup fields, and repeated
  `Vec<Markup>` item setters

Current limits:
- named structs only
- at most one `#[slot(default)]` field
- `.build()` is still required
- there is no `compose!` macro or block syntax yet
- the builder itself doesn't implement `Render`

## Runtime Slots

The runtime slot API is still supported, but it is no longer the aspirational
surface for new shell and layout components.

Why it exists:
- it works for fully open caller-owned child structure
- it keeps existing slot-based components working
- it provides one generic transport path over plain `Render`

Why it is lower-level:
- slot names are stringly
- child transport goes through `.with_children(...)`
- the slot contract stays outside the type system
- missing or extra named slots are runtime behavior, not builder-shape errors

Use it when openness is the point. Otherwise prefer `ComponentBuilder`.

## Signals

Signals support is intentionally JS-first. Render stable DOM anchors in Maud,
then create signals and bindings in `js!`.

```rust
use maud::{html, Markup, Render};
use maud_extensions::{component, css, js, surreal_scope_signals_inline};

js! {
    const count = mx.signal(0);
    const active = mx.computed(() => count.value > 0);

    me(".count").bindText(count);
    me().bindClass("active", active);
    me(".inc").on("click", () => count.value++);
}

struct Counter;

impl Render for Counter {
    fn render(&self) -> Markup {
        component! {
            @js-once
            section class="counter" {
                p { "Count: " span class="count" {} }
                button class="inc" type="button" { "+" }
            }
        }
    }
}

css! {
    me.active { border-color: #16a34a; }
}

let page = html! {
    head { (surreal_scope_signals_inline!()) }
    body { (Counter) }
};
```

Supported v1 binders:
- `bindText(source)`
- `bindAttr(name, source)`
- `bindClass(name, source)`
- `bindShow(source)`

Rules:
- binders live on `window.mx` and on Surreal-sugared handles such as
  `me(".count")`
- `source` can be a Signals object or a function
- function sources run inside `mx.effect(...)`
- binder cleanup is scoped to `component!` roots
- `surreal_scope_signals_inline!()` is the supported runtime include when a
  page uses `component!`, `js!`, and Signals binders together

## Runtime And Direct Emitters

Bundled runtime macros:
- `surreal_scope_inline!()`
  - emits bundled `surreal.js` and `css-scope-inline.js`
- `signals_inline!()`
  - emits bundled `@preact/signals-core` and the Maud Signals adapter
- `surreal_scope_signals_inline!()`
  - emits `surreal.js`, `css-scope-inline.js`, `@preact/signals-core`, and
    the Maud Signals adapter in the right order

Direct emitters:
- `inline_js! { ... }` / `inline_css! { ... }`
  - emit direct `<script>` / `<style>` tags
- `js_file!(...)` / `css_file!(...)`
  - inline file contents using `include_str!`-style paths
- `font_face!(...)` / `font_faces!(...)`
  - emit base64 `@font-face` CSS without adding another dependency

Manual composition rule:
- if you emit `surreal_scope_inline!()` and `signals_inline!()` separately,
  put `surreal_scope_inline!()` first so the Signals adapter can extend
  Surreal before component `js!` blocks run

## Limits And Guarantees

- `component!` performs compile-time shape checks over the token stream it
  sees; it only checks the token shape the macro can observe
- `component!` accepts exactly one top-level element with a body block
- `js!` and `css!` must both be in scope for `component!`, even if one is empty
- `inline_js!` validates JavaScript with SWC before generating markup
- `inline_css!` runs a lightweight CSS syntax check before generating markup
- slot runtime helpers fail closed outside `.with_children(...)`
- malformed slot transport markers fail closed into default-slot content
- runtime slots are a lower-level transport layer; for new shell/layout
  components prefer `ComponentBuilder`
- `ComponentBuilder` observes lexical type forms, not full type resolution, so
  type aliases to `Markup`, `Option`, or `Vec` are treated as ordinary fields

## Read Next

- [examples/component_card.rs](examples/component_card.rs)
- [examples/signals_counter.rs](examples/signals_counter.rs)
- [examples/runtime_injection.rs](examples/runtime_injection.rs)
- [examples/slots.rs](examples/slots.rs)
- [tests/component_builder.rs](tests/component_builder.rs)
- [docs.rs for `maud-extensions`](https://docs.rs/maud-extensions)
- [docs.rs for `maud-extensions-runtime`](https://docs.rs/maud-extensions-runtime)

## License

MIT OR Apache-2.0
