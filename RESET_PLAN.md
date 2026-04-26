# mx reset plan

This worktree exists to explore a smaller, more Maud-native core surface for
`maud-extensions` / `mx`.

## Problem statement

The current crate had accumulated several overlapping stories:

- plain Maud composition
- direct emitters
- typed builders
- runtime slots
- scoped CSS runtime behavior
- CSS helper DSL growth

That makes the product feel more framework-like than intended.

## Reset goal

Return to a surface where the default workflow is just:

1. write normal `html!`
2. insert local CSS/JS as markup where they belong
3. optionally name those local CSS/JS emitters when reuse helps

The library should feel like **tiny Maud superpowers**, not a replacement for
Maud.

## Proposed core public story

### Beautiful default

```rust
maud::html! {
    article.card {
        h2 { "Title" }
        p { "Body" }

        (mx::css! {
            me { padding: 1rem; }
        })

        (mx::js! {
            me().class_add("ready");
        })
    }
}
```

### Optional named helper form

```rust
mx::css!(card_css, {
    me { padding: 1rem; }
});

mx::js!(card_js, {
    me().class_add("ready");
});
```

### Optional explicit JS mode

```rust
mx::js!(once, {
    me().class_add("ready");
});

mx::js!(card_js, once, {
    me().class_add("ready");
});
```

## First-class workflows

1. **Plain Maud first**
   - Use ordinary `html!` whenever possible.

2. **Local CSS/JS emitters**
   - `css!` and `js!` should return `Markup` in the common case.
   - Placement should stay explicit and local.

3. **Direct emitters**
   - `inline_css!`, `inline_js!`, `css_file!`, `js_file!`, `font_face!`,
     `font_faces!`, and runtime include macros remain useful.

4. **Advanced composition later**
   - Typed builders and slots may remain, but they are not the default story.

## Removed from the reset core story

- `component!`
- hidden CSS/JS helper injection
- directive-based JS mode on a separate wrapper macro
- stringly naming/configuration forms

## Design rules

1. **No hidden injection in the default story**
   - users should see where CSS and JS are emitted in `html!`

2. **Non-stringy API where possible**
   - prefer `css!(card_css, { ... })` over `css!("card_css", { ... })`
   - prefer `js!(once, { ... })` over string flags

3. **Maud remains the main language**
   - avoid adding a second component language on top

4. **Locality over magic**
   - CSS/JS ownership should be obvious from placement in markup

5. **Smallest honest surface first**
   - get the inline unnamed form right before preserving every old feature

## First implementation slice

The first slice should be intentionally narrow:

1. Keep current crate compiling and tests passing.
2. Add a new parsing path for:
   - `css! { ... } -> Markup`
   - `js! { ... } -> Markup`
3. Add a new named helper form:
   - `css!(ident, { ... })`
   - `js!(ident, { ... })`
4. Add explicit JS once mode:
   - `js!(once, { ... })`
   - `js!(ident, once, { ... })`
5. Remove legacy `component!`-centric tests/docs once the new core surface is real.

## Migration boundary for the first slice

In this reset branch, decisions should be driven by the reset story rather than
by preserving every old abstraction as first-class.

## Questions to answer during implementation

1. Should the new named forms generate local helper functions exactly like the
   current named `css!` helper path, or should they use a different internal
   model?
2. Should `js! { ... }` default to always-run semantics in inline form, with
   `once` as an explicit opt-in?
3. How much of the current CSS helper DSL should remain in the reset story?
   Likely answer: keep the structured helpers already added, but do not expand
   the DSL further until the core surface stabilizes.

## Success criteria

We should be able to say, truthfully:

> The default way to use `mx` is to write plain `html!` and drop in local
> `css!` / `js!` emitters where they belong.
