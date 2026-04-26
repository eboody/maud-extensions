The key is:

- **plain Maud remains valid**
- but if you want the premium component authoring experience, you opt into it
- and the derive gives you a **builder-centric, zero-regret API**

---

# Vision

```rust
#[derive(mx::Component)]
struct Card<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    tone: Tone,
    body: mx::Slot,
    actions: mx::Slot,
}
```

becomes:

- a typed builder
- typed slot setters
- optional/default ergonomics
- repeated/named child ergonomics
- render support
- local CSS/JS composition hooks
- beautiful compile-time diagnostics

The **struct is the truth**.  
The macro turns that truth into the nicest possible component API.

---

# Core philosophy

## 1. Struct-first
The struct is the canonical declaration of:

- props
- children
- slots
- defaults
- optionality
- repeatability
- maybe local assets

Not a separate DSL.

## 2. Builder is the primary devex
The output should feel like:

```rust
Card::new()
    .title("Account")
    .subtitle("Billing details")
    .body(html! { "..." })
    .action(button("Save"))
    .render()
```

not like ceremonial Rust plumbing.

## 3. Maud stays the rendering language
Rendering should still be ordinary:

- `impl Render`
- or generated render hook using `html!`

No replacement templating syntax required.

## 4. Slots/props are encoded in types
Missing required stuff should fail at compile time where reasonable.

---

# Ideal user-facing API

## Minimal case

```rust
#[derive(mx::Component)]
struct Badge<'a> {
    label: &'a str,
}
```

Use:

```rust
Badge::new().label("New").render()
```

Generated:

- `Badge::new()`
- typestate builder
- `.build()`
- maybe `.render()` shortcut

---

## Optional props

```rust
#[derive(mx::Component)]
struct Alert<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
}
```

Use:

```rust
Alert::new()
    .title("Heads up")
    .maybe_subtitle(Some("Something changed"))
    .render()
```

or:

```rust
Alert::new()
    .title("Heads up")
    .subtitle("Something changed")
    .render()
```

Ideal behavior:
- optional fields get both:
  - direct setter
  - `maybe_*` setter

---

## Defaulted props

```rust
#[derive(mx::Component)]
struct Button<'a> {
    label: &'a str,
    #[mx(default = Tone::Primary)]
    tone: Tone,
    #[mx(default = Size::Md)]
    size: Size,
}
```

Use:

```rust
Button::new().label("Save").render()
Button::new().label("Delete").tone(Tone::Danger).render()
```

---

## Default body slot

```rust
#[derive(mx::Component)]
struct Card<'a> {
    title: &'a str,
    #[mx(slot, default)]
    body: maud::Markup,
}
```

Use:

```rust
Card::new()
    .title("Settings")
    .body(html! { p { "Body" } })
    .render()
```

But ideally also:

```rust
Card::new()
    .title("Settings")
    .child(html! { p { "Body" } })
    .render()
```

or even:

```rust
Card::new()
    .title("Settings")
    .children(html! { p { "Body" } })
    .render()
```

For the default slot, the builder should feel frictionless.

---

## Named slots

```rust
#[derive(mx::Component)]
struct Panel<'a> {
    title: &'a str,
    #[mx(slot)]
    header: Option<maud::Markup>,
    #[mx(slot, default)]
    body: maud::Markup,
    #[mx(slot)]
    footer: Option<maud::Markup>,
}
```

Use:

```rust
Panel::new()
    .title("Account")
    .header(html! { h2 { "Profile" } })
    .body(html! { p { "Body" } })
    .footer(html! { button { "Save" } })
    .render()
```

For optional named slots:
- slot setter should accept `Render`, not just `Markup`
- maybe also `.maybe_header(...)`

---

## Repeated slots / collections

```rust
#[derive(mx::Component)]
struct Menu<'a> {
    label: &'a str,
    #[mx(slot, each = item)]
    items: Vec<maud::Markup>,
}
```

Use:

```rust
Menu::new()
    .label("Actions")
    .item(html! { li { "Edit" } })
    .item(html! { li { "Archive" } })
    .render()
```

Also support bulk setter:

```rust
.items(vec![...])
```

Ideal repeated ergonomics are huge.

---

# Ideal field kinds

The derive should understand these kinds semantically.

## Plain prop
```rust
title: &'a str
```

Required unless defaulted/optional.

## Optional prop
```rust
subtitle: Option<&'a str>
```

Semantically optional.

## Repeated prop
```rust
items: Vec<Item>
```

Potentially `each = item`.

## Default slot
```rust
#[mx(slot, default)]
body: Markup
```

Primary child content.

## Named optional slot
```rust
#[mx(slot)]
header: Option<Markup>
```

Named region.

## Named repeated slot
```rust
#[mx(slot, each = action)]
actions: Vec<Markup>
```

Repeated child region.

## Maybe richer slot types
Ideally not only `Markup`, but maybe:
- anything implementing `Render`
- normalized into markup internally

---

# Incredible builder ergonomics

## 1. `.render()` directly on the builder
This is huge.

Instead of:

```rust
Card::new().title("X").body(html! {...}).build().render()
```

just:

```rust
Card::new().title("X").body(html! {...}).render()
```

And `.build()` still exists if you need the concrete value.

## 2. Accept `impl Render` almost everywhere
If I pass a child/slot, I don’t want to constantly write `html!`.

I want:

```rust
.header(MyHeading { ... })
.action(Button::new().label("Save"))
```

Anything renderable should just work.

## 3. String-friendly setters
For common string props:

```rust
title: String
```

setter should accept:
- `String`
- `&str`
- maybe `Cow<'a, str>` patterns where appropriate

So basically automatic `Into`.

## 4. Enum-friendly toggles
If a field is bool-ish or enum-ish, maybe generate convenience helpers.

Example:

```rust
#[derive(mx::Component)]
struct Button<'a> {
    label: &'a str,
    #[mx(default = false)]
    disabled: bool,
}
```

Maybe:

```rust
.disabled(true)
.enable()
.disable()
```

Maybe too magical by default, but ideal-world ergonomic.

## 5. Nice optional setter patterns
For `Option<T>`:
- `.subtitle("x")`
- `.maybe_subtitle(opt)`

For `Vec<T>`:
- `.item(x)`
- `.items(iterable)`

For default slot:
- `.child(x)`
- `.children(x)`

---

# Ideal rendering story

There are two paths.

## Path A: derive only the builder
User writes `impl Render`.

This is the safest and probably best default.

```rust
#[derive(mx::Component)]
struct Card<'a> { ... }

impl Render for Card<'_> {
    fn render(&self) -> Markup { ... }
}
```

## Path B: optional render macro integration
Maybe later:

```rust
#[mx::render]
impl Render for Card<'_> {
    fn render(&self) -> Markup {
        html! { ... }
    }
}
```

or some helper for colocated assets.

But I would keep the first release focused on builder ergonomics.

---

# Ideal slot normalization

One of the biggest wins would be if slots didn’t force raw `Markup` everywhere.

Imagine a special internal type:

```rust
mx::Slot
mx::Slots
```

So the user can write:

```rust
#[derive(mx::Component)]
struct Card<'a> {
    title: &'a str,
    #[mx(slot, default)]
    body: mx::Slot,
}
```

And the setter accepts:
- `Markup`
- anything `Render`
- maybe iterables for repeated slots

This could be much nicer than exposing raw `Markup` as the semantic child type.

That would be a big devex improvement.

---

# Ideal compile-time diagnostics

This is a huge part of the premium experience.

## Missing required prop
Error should say:

- component name
- missing field
- how to set it

Example:

> `Card::build()` requires `title` and `body`; `title` is a required prop and `body` is the default slot.

## Duplicate/default slot conflicts
Should explain the semantic rule, not macro gibberish.

## Bad `each` usage
If someone puts `#[mx(each = item)]` on a non-collection field, tell them exactly that.

## Reserved method name collisions
If a field would generate a bad builder method, error clearly and suggest rename/override.

---

# Ideal advanced features

## 1. Rename builder methods
```rust
#[mx(setter = heading)]
title: &'a str,
```

## 2. Explicit child aliases
```rust
#[mx(slot, default, setter = child)]
body: mx::Slot,
```

## 3. Slot grouping
Maybe:

```rust
#[mx(slot, group = "actions", each = action)]
actions: Vec<mx::Slot>,
```

Not necessary at first, but could be great.

## 4. Generic/renderable prop coercion
Props that conceptually accept content should be able to take renderables directly.

## 5. Builder presets / variants
Maybe later:

```rust
Button::primary("Save")
Button::danger("Delete")
```

Generated from metadata.

---

# Local CSS/JS integration dream

This is where it gets really fun.

Imagine component-local asset hooks declared on the type:

```rust
#[derive(mx::Component)]
#[mx(css = card_css, js(once) = card_js)]
struct Card<'a> {
    title: &'a str,
    #[mx(slot, default)]
    body: mx::Slot,
}
```

Then generated helpers or conventions for embedding those assets inside render become easy.

Or maybe simpler:

```rust
impl Card<'_> {
    fn styles() -> Markup { css! { ... } }
    fn scripts() -> Markup { js!(once, { ... }) }
}
```

And the derive could optionally know how to wire them in.

I would not start here, but as a dream devex it’s great.

---

# Ideal “one true experience”

The perfect-feeling component authoring flow might be:

```rust
#[derive(mx::Component)]
struct Card<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    #[mx(slot, default)]
    body: mx::Slot,
    #[mx(slot, each = action)]
    actions: Vec<mx::Slot>,
}

impl Render for Card<'_> {
    fn render(&self) -> Markup {
        html! {
            article.card {
                h2 { (self.title) }
                @if let Some(subtitle) = self.subtitle {
                    p.subtitle { (subtitle) }
                }
                div.body { (self.body) }
                @if !self.actions.is_empty() {
                    footer.actions {
                        @for action in &self.actions {
                            (action)
                        }
                    }
                }
            }
        }
    }
}
```

Use:

```rust
Card::new()
    .title("Settings")
    .subtitle("Manage your account")
    .child(html! { p { "Profile details" } })
    .action(Button::new().label("Save"))
    .action(Button::new().label("Cancel"))
    .render()
```

That’s really, really good.

---

## Must-have
- typed builder from struct
- required/optional/default fields
- `.render()` on builder
- slots
- default slot ergonomics
- repeated slot ergonomics
- accept `impl Render` for slot/content setters
- excellent diagnostics

## Very desirable
- setter renaming
- maybe-setters
- each-setters
- coercions via `Into`
- semantic slot wrapper types like `mx::Slot`

## Nice later
- builder presets
- asset integration
- component-local conventions
- convenience constructors

---

# Addendum: Bon-enabled advanced possibilities

These ideas come specifically from Bon's more advanced and experimental surface.
They do not change the core philosophy; they expand how premium the
builder-centric experience could become.

## 1. Generic slot specialization

Bon's experimental generic-parameter setters suggest a very powerful content
story for components that begin with empty or placeholder child content and
become concrete later in the builder chain.

That could enable a model like:

```rust
#[derive(mx::Component)]
struct Card<Body = ()> {
    title: String,
    body: Body,
}
```

with a user experience where:

- `Card::new().title("Settings").build()` means empty/default body
- `Card::new().title("Settings").child(html! { ... }).build()` rewrites the
  body generic to a concrete content type

This is exciting because it could make default-slot ergonomics feel dynamic at
the callsite while staying type-driven underneath.

## 2. Generated custom semantic methods over a strict typestate core

Bon's typestate API means `mx::Component` would not be limited to default
generated setters. The derive could generate hand-crafted semantic sugar while
still preserving compile-time guarantees.

Examples:

- `.child(...)`
- `.children(...)`
- `.header(...)`
- `.footer(...)`
- `.action(...)`
- `.render()`
- grouped setters like `.title_and_subtitle(...)`

This means the public component API can feel bespoke and beautiful while Bon
still provides the actual safety machinery.

## 3. Hidden low-level builder, curated high-level component API

Bon's naming and visibility controls suggest a particularly strong design: keep
raw builder internals private or obscure, and expose only the semantic methods
that make sense for components.

The user experience should feel like:

```rust
Card::new()
    .title("Settings")
    .child(html! { p { "Profile details" } })
    .action(Button::new().label("Save"))
    .render()
```

not like interacting with a generic builder framework.

In other words, Bon can be the engine, while `mx::Component` is the authored
component UX.

## 4. Type-wide policy application for ergonomic defaults

Bon's `on(...)` support suggests `mx::Component` could apply broad ergonomic
policies automatically, rather than requiring lots of repetitive per-field
attributes.

Examples of policies that may make sense internally:

- all `String` props accept `&str` / `impl Into<String>`
- all repeated collections get both bulk and item-level setters
- all optional props get `maybe_*` setters
- selected slot/content shapes get custom semantic aliases

This matters because it lets the derive feel coherent instead of attribute-heavy.

## 5. Fixture and preset flows inspired by overwritable builders

Bon's experimental `overwritable` feature is probably not right as the default
for production component builders, because repeated setter calls are often bugs.

However, it suggests a useful adjacent idea:

- fixture-oriented builders for tests
- preview/story/demo builders
- preset layering and theme composition

So while `overwritable` should likely stay out of the normal `mx::Component`
contract, it may inspire optional fixture or preview modes later.

## 6. Component-native start and finish surfaces

Bon's configurable start/finish functions mean the derive can present a very
intentional API shape.

Examples:

- `Component::new()` as the canonical start
- `.build()` when the concrete value is wanted
- `.render()` as the premium happy path
- hidden or renamed raw internals when necessary

This is important because the component authoring experience should feel native
to `mx`, not obviously like raw Bon terminology leaking through.

## 7. Rich slot/content normalization without exposing the machinery

Bon's custom methods plus typestate suggest a path where slot setters accept a
wide range of inputs:

- `Markup`
- anything implementing `Render`
- maybe iterables for repeated content

while normalizing all of that into internal slot representations.

This reinforces a key aspirational idea: callers should not have to constantly
think in terms of raw `Markup` if the semantic operation is "provide child
content" or "append an action".

## 8. Product philosophy still stays the same

Even with these advanced powers available, the guiding philosophy should remain:

- the struct is the truth
- Maud is still the rendering language
- the derive improves construction, not replaces rendering
- the magic should compress ceremony, not invent a second UI framework

So Bon expands implementation power much more than it changes the product's
identity.
