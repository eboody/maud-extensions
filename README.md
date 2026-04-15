# maud-extensions

[![crates.io](https://img.shields.io/crates/v/maud-extensions.svg)](https://crates.io/crates/maud-extensions)
[![docs.rs](https://img.shields.io/docsrs/maud-extensions)](https://docs.rs/maud-extensions)
[![license](https://img.shields.io/crates/l/maud-extensions.svg)](https://github.com/eboody/maud-extensions)

Proc macros for Maud that make inline CSS/JS, component-style authoring, and
small bundled browser runtimes simpler.

This crate includes bundled copies of
[gnat/surreal](https://github.com/gnat/surreal),
[gnat/css-scope-inline](https://github.com/gnat/css-scope-inline), and an
optional bundled copy of
[`@preact/signals-core`](https://github.com/preactjs/signals). Signals support
stays JS-first: Maud markup provides anchors, while `js!` owns signals and DOM
bindings.

## Why use it?
- Define component-local `js()` and `css()` helpers with `js!` / `css!`.
- Wrap markup with `component!` and auto-inject JS/CSS helpers.
- Add component lifecycle behavior with `@js-once` / `@js-always`.
- Emit direct `<script>` / `<style>` blocks when needed.
- Bundle `surreal.js` and `css-scope-inline.js` with zero path setup.
- Add client-side Signals binders without a JS bundler.
- Embed fonts as base64 `@font-face` CSS.

## Table of Contents
- [Install](#install)
- [Support Policy](#support-policy)
- [Guarantees and Limits](#guarantees-and-limits)
- [What's New in 0.4.x](#whats-new-in-04x)
- [Quick Start](#quick-start)
- [component!](#component)
- [Lifecycle Cleanup](#lifecycle-cleanup)
- [Signals](#signals)
- [Slots](#slots)
- [Runtime Injection](#runtime-injection)
- [Macro Reference](#macro-reference)
- [Runtime Slot API](#runtime-slot-api)
- [Font Helpers](#font-helpers)
- [Migration Guide (0.3 -> 0.4)](#migration-guide-03---04)
- [Migration Guide (0.2 -> 0.3)](#migration-guide-02---03)
- [License](#license)

## Install

```bash
cargo add maud-extensions
cargo add maud-extensions-runtime # needed for slots + `.in_slot("name")`
```

## Support Policy

- MSRV: Rust 1.85
- Supported Maud version: 0.27
- CI runs the crate on stable and the MSRV

## Guarantees and Limits

- `component!` accepts one top-level Maud element with a body block.
- `component!` shape checks happen at compile time over the token stream the macro sees. It isn't a full Maud parser.
- `inline_js!` parses emitted JavaScript with `swc_ecma_parser` before it generates markup.
- `inline_css!` runs a lightweight CSS syntax check before it generates markup.
- Signals binders are JS-first. Use markup for anchors and `js!` for signals, effects, and DOM binding.
- Signals binders fail closed when the target is outside a `component!` root, because automatic cleanup is scoped to component roots.
- `slot()` and `named_slot("...")` return empty markup outside `.with_children(...)`.
- duplicate named slots are concatenated in render order.
- malformed slot transport markers fail closed into default slot content instead of being partially consumed.
- `js_file!` / `css_file!` accept paths that work with `include_str!`.
- `font_face!` / `font_faces!` accept paths that work with `include_bytes!`.

## What's New in 0.4.x

- New `component!` macro for auto-injecting JS/CSS helpers into one root element.
- Swapped JS/CSS macro naming so `js!`/`css!` define local helpers and
  `inline_js!`/`inline_css!` emit direct tags.
- Bundled runtime helper `surreal_scope_inline!()` with no path setup.
- Explicit compile-time shape checks for `component!` input.
- Optional `component!` JS mode directives: `@js-once` and `@js-always`.
- Component-scoped JS cleanup helpers in bundled `surreal.js`.
- Slot flow simplified to runtime APIs: `slot()`, `named_slot("...")`, and
  `.with_children(...)` + `.in_slot("...")`.

## Quick Start

This example shows the single-file component pattern: `js!` at the top,
`component!` markup in `Render`, and `css!` at the bottom.

Compile-tested versions of the core workflows live in [`examples/`](examples).

```rust
use maud::{html, Markup, Render};
use maud_extensions::{component, css, js, surreal_scope_inline};

// Component behavior at file scope.
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

// Component styles at file scope.
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

`component!` wraps one top-level Maud element and appends the JS/CSS helpers
generated by `js!` and `css!` inside that root element automatically.

```rust
use maud::{Markup, Render};
use maud_extensions::{component, css, js};

js! {
    me().class_add("ready");
}

struct Card;

impl Render for Card {
    fn render(&self) -> Markup {
        component! {
            section class="card" {
                p { "Hello" }
            }
        }
    }
}

css! {
    me { border: 1px solid #ddd; }
}
```

Equivalent output shape:
- root element content
- then `(js())`
- then `(css())`

Rules:
- optional directives are supported before the root element:
  `@js-once` or `@js-always`
- input must be exactly one top-level element with a `{ ... }` body
- `js! { ... }` and `css! { ... }` must be present in scope (empty is valid: `js! {}` / `css! {}`)
- a clean pattern is one component per module/file with `js!` above and `css!` below the `Render` impl
- trailing `;` is allowed
- invalid root shapes fail at compile time with guidance
- if a helper is missing, the compiler error points at a required internal helper symbol;
  add the corresponding `js! { ... }` or `css! { ... }` call
- `component!` roots include `data-mx-component` and `data-mx-js-mode`
  attributes for runtime lifecycle behavior

## Lifecycle Cleanup

When `surreal_scope_inline!()` is present, component roots can register cleanup
work and auto-track common side effects.

```rust
use maud::{Markup, Render};
use maud_extensions::{component, css, js};

js! {
    const onResize = () => me().class_add("resized");
    onWindow("resize", onResize);

    const tick = interval(() => me().class_add("ping"), 1000);
    me().cleanup(() => clearInterval(tick));

    const observer = observeMutations(me(), () => {});
    me().cleanup(() => observer && observer.disconnect());

    // Auto-tracked: removed when the component root unmounts.
    me("button").on("click", () => me().class_add("clicked"));
}

struct LifecycleDemo;

impl Render for LifecycleDemo {
    fn render(&self) -> Markup {
        component! {
            @js-once
            section class="lifecycle-demo" {
                button { "Click me" }
            }
        }
    }
}

css! {}
```

Notes:
- `@js-once` runs component JS once per root element.
- `@js-always` (default) runs JS each time the script executes.
- cleanup ownership is scoped to `component!` roots.
- helpers available from bundled `surreal.js` include:
  `cleanup`, `onWindow`, `onDocument`, `timeout`, `interval`,
  and `observeMutations`.

## Signals

Signals support is intentionally JS-first. Use Maud markup to render stable
DOM anchors, then create signals and bind them in `js!`.

```rust
use maud::{Markup, Render, html};
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
- `source` can be a Signals object like `mx.signal(...)` / `mx.computed(...)`, or a function.
- function sources run inside `mx.effect(...)`, so dependencies are tracked automatically.
- binders are exposed on `window.mx` and also added to Surreal-sugared elements like `me(".count")`.
- component cleanup owns the binder effects, so bindings stop when the `component!` root leaves the DOM.

## Slots

Use runtime slot functions inside your component template, then pass children
through `.with_children(...)`. Unannotated children go to the default slot, and
named content is tagged with `.in_slot("name")`.

```rust
use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

struct Card;

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class="card" {
                header class="card-header" { (named_slot("header")) }
                section class="card-body" { (slot()) }
            }
        }
    }
}

struct CardHeader<'a> {
    title: &'a str,
}

impl<'a> Render for CardHeader<'a> {
    fn render(&self) -> Markup {
        html! { h2 { (self.title) } }
    }
}

let view = html! {
    (Card.with_children(html! {
        (CardHeader { title: "Status" }.in_slot("header"))
        p { "All systems operational" }
    }))
};
```

Rules:
- `slot()` renders the default slot.
- `named_slot("name")` renders a named slot.
- `.in_slot("name")` assigns a child component to that named slot.
- `.with_children(html! { ... })` provides child content for slot resolution.
- missing named slots render empty content.
- extra provided named slots are ignored.

## Runtime Injection

Use bundled runtime scripts with no filesystem setup:

```rust
use maud_extensions::surreal_scope_inline;

maud::html! {
    (surreal_scope_inline!())
}
```

Signals only:

```rust
use maud_extensions::signals_inline;

maud::html! {
    (signals_inline!())
}
```

Full component stack with Signals binders:

```rust
use maud_extensions::surreal_scope_signals_inline;

maud::html! {
    (surreal_scope_signals_inline!())
}
```

If you compose the runtime macros manually, emit `surreal_scope_inline!()`
before `signals_inline!()` so the Signals adapter can patch Surreal before
component `js!` blocks run.

Need custom files instead? Use `js_file!` / `css_file!` (`include_str!` behavior):

```rust
use maud_extensions::js_file;

maud::html! {
    (js_file!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/vendor/custom-runtime.js")))
}
```

## Macro Reference

- `js! { ... }` / `js!("...")`
  - Generate local `fn js() -> maud::Markup` and the hidden helper used by `component!`.
- `css! { ... }` / `css!("...")`
  - Generate local `fn css() -> maud::Markup` and the hidden helper used by `component!`.
- `component! { ... }`
  - Wrap one root element and inject helpers emitted by `js!` / `css!` at the end of its body.
  - Supports optional top directives: `@js-once`, `@js-always`.
- `inline_js! { ... }` / `inline_js!("...")`
  - Emit `<script>` markup directly.
  - Validate JS via `swc_ecma_parser`.
- `inline_css! { ... }` / `inline_css!("...")`
  - Emit `<style>` markup directly.
  - Run a lightweight CSS syntax check via `cssparser`.
- `js_file!("path")` / `css_file!("path")`
  - Emit `<script>` / `<style>` tags from file contents accepted by `include_str!`.
- `surreal_scope_inline!()`
  - Emit bundled `surreal.js` and `css-scope-inline.js`.
- `signals_inline!()`
  - Emit bundled `@preact/signals-core` plus the Maud Signals adapter.
- `surreal_scope_signals_inline!()`
  - Emit bundled `surreal.js`, `css-scope-inline.js`, `@preact/signals-core`, and the Maud Signals adapter.
- `font_face!(...)` / `font_faces!(...)`
  - Embed font files as base64 `@font-face` CSS.

## Runtime Slot API

From `maud-extensions-runtime`:

- `prelude::*`
  - Re-exports `slot`, `named_slot`, `WithChildrenExt`, and `InSlotExt`.

- `slot()`
  - Render default slot content for the current slotted component context.
- `named_slot("name")`
  - Render named slot content.
- `WithChildrenExt::with_children(html! { ... })`
  - Attach child content to a component value before rendering.
- `InSlotExt::in_slot("name")`
  - Mark child content for a named slot.

## Font Helpers

`font_face!` and `font_faces!` embed font files as base64 data URLs without
requiring an extra dependency in the consuming crate.

```rust
use maud_extensions::font_face;

maud::html! {
    (font_face!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/demo-font.woff2"),
        "JetBrains Mono"
    ))
}
```

## Migration Guide (0.3 -> 0.4)

### 1. Replace slot macros with runtime functions

Old:

```rust
use maud_extensions::slot;

html! {
    (slot!())
    (slot!("header"))
}
```

New:

```rust
use maud_extensions_runtime::prelude::*;

html! {
    (slot())
    (named_slot("header"))
}
```

### 2. Replace `use_component!` with `.with_children(...)`

Old:

```rust
use maud_extensions::use_component;
use maud_extensions_runtime::prelude::*;

html! {
    (use_component!(
        Card,
        {
            (Title.in_slot("header"))
            p { "Body" }
        }
    ))
}
```

New:

```rust
use maud_extensions_runtime::prelude::*;

html! {
    (Card.with_children(html! {
        (Title.in_slot("header"))
        p { "Body" }
    }))
}
```

## Migration Guide (0.2 -> 0.3)

### 1. Rename JS/CSS macro usage

The JS/CSS macro names were intentionally swapped:

- old `js!` -> new `inline_js!`
- old `css!` -> new `inline_css!`
- old `inline_js!` -> new `js!`
- old `inline_css!` -> new `css!`

### 2. Move to `component!` for root injection

Old pattern:

```rust
inline_js! { me().class_add("ready"); }
let view = maud::html! {
    article {
        "Hello"
        (js())
        (css())
    }
};
inline_css! { me { color: red; } }
```

New pattern:

```rust
js! { me().class_add("ready"); }
let view = component! {
    article {
        "Hello"
    }
};
css! { me { color: red; } }
```

### 3. Keep runtime scripts explicit in layout/page shell

```rust
maud::html! {
    head {
        (surreal_scope_inline!())
    }
}
```

### 4. Update assumptions in your codebase

- `component!` requires exactly one top-level element with a body block.
- `component!` expects `js!` and `css!` calls in scope (empty blocks are allowed).
- defining `fn js()` / `fn css()` manually isn't enough; use `js!` / `css!` so `component!` sees required helpers.
- `font_face!`/`font_faces!` embed data URLs without an extra dependency in the consuming crate.
- `js_file!`/`css_file!` paths follow `include_str!` behavior.

## License

MIT OR Apache-2.0
