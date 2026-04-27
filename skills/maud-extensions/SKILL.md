# Skill: maud-extensions

Use this skill when designing, extending, or reviewing the `maud-extensions`
component system and its colocated CSS/JS story.

## Core philosophy

`maud-extensions` should make Maud feel like it has small, local superpowers.
It should not replace Maud with a second framework-shaped abstraction unless the
new surface is more honest and ergonomic than plain Maud.

For components specifically:

- the component struct is the semantic source of truth for props and slots
- slot-ness should live in the field type (`Slot<T>`, `Slot<Vec<T>>`)
- Bon owns builder mechanics and typestate completion
- CSS and JS should feel component-owned and colocated
- rendering should stay explicit and readable
- builder `.render()` should just render the completed component value

## Preferred component surface

Today the preferred experimental authoring pattern is:

1. `#[derive(Component)]` on the struct
2. `Slot<Markup>` / `Slot<Vec<Markup>>` for slots
3. `#[mx(default)]` for the single default slot
4. `#[mx(each = item_name)]` for repeated slot item setters
5. explicit `fn css() -> Markup` and `fn js() -> Markup` helpers when needed
6. ordinary `impl Render` with explicit `(Self::css())` / `(Self::js())`

Important: component authors should not need extra macro-owned render glue.

## The thought process for building a component

### 1. Start with the data contract

Ask:

- what are the props?
- what are the child regions?
- which child region is the default?
- which regions repeat?

Write the struct first.

```rust
#[derive(mx::Component)]
struct Card {
    title: String,
    header: Slot<Markup>,
    #[mx(default)]
    body: Slot<Markup>,
    actions: Slot<Vec<Markup>>,
}
```

The struct is the truth.

### 2. Let the field types declare meaning

Think:

- plain fields are props
- `Slot<Markup>` is one child region
- `Slot<Vec<Markup>>` is a repeated child region
- `#[mx(default)]` marks the unnamed/default child slot
- `#[mx(default)]` is reserved for slot selection only; use Rust `Default` or
  `Option<T>` for non-slot defaults

### 3. Think in builder terms

You want:

```rust
Card::new()
    .title("Profile")
    .header(html! { span { "Welcome" } })
    .child(html! { p { "Body" } })
    .action(html! { button { "Save" } })
    .action(html! { button { "Cancel" } })
```

So the builder should reflect the component’s semantic structure:

- prop setters
- default slot alias via `.child(...)`
- repeated slot item setters via `#[mx(each = action)]`

### 4. Define explicit local CSS helpers only if the component owns styles

```rust
impl Card {
    fn css() -> Markup {
        mx::css! {
            me {
                padding: 1rem;
                border: 1px solid #ddd;
            }

            me .actions {
                display: flex;
                gap: 0.5rem;
            }
        }
    }
}
```

### 5. Define explicit local JS helpers only if the component owns behavior

```rust
impl Card {
    fn js() -> Markup {
        mx::js!(once, {
            me().class_add("ready");
        })
    }
}
```

### 6. Define the render root explicitly

```rust
impl Render for Card {
    fn render(&self) -> Markup {
        maud::html! {
            article.card {
                (Self::css())
                (Self::js())
                header class="header" { (self.header) }
                h2 { (self.title) }
                div.body { (self.body) }
                div.actions { (self.actions) }
            }
        }
    }
}
```

This keeps the render tree honest and makes asset placement obvious.

### 7. Let the builder render the completed component

Now the builder `.render()` should mean:

- build the complete component
- call the ordinary `Render` impl
- return final markup

### 8. Bootstrap the browser runtime once per page

If the page relies on Surreal / css-scope-inline / Signals, prefer:

```rust
html! {
    head {
        (mx::Init::all())
    }
    body {
        (Card::new().title("Profile").render())
    }
}
```

## Browser-side building blocks

The current component-local CSS/JS story layers on top of:

- Surreal: <https://github.com/gnat/surreal>
- css-scope-inline: <https://github.com/gnat/css-scope-inline>
- Preact Signals: <https://github.com/preactjs/signals>

## Current guardrails

- tuple/unit/unnamed-field structs are invalid component declarations
- legacy slot attrs are rejected
- repeated item setters require repeated slot storage
- `#[mx(default)]` is reserved for slots

## Review checklist

When changing this component system, prefer solutions that keep these truths
intact:

- Bon remains the builder engine
- the slot declaration path stays type-driven
- component-local CSS/JS stay colocated
- rendering stays explicit and easy to reason about
- component authors do not need boilerplate beyond plain Rust helper methods
- the public model stays smaller and more honest than a framework clone
