# Skill: maud-extensions

Use this skill when designing, extending, or reviewing the `maud-extensions`
 component system and its colocated CSS/JS story.

## Core philosophy

`maud-extensions` should make Maud feel like it has small, local superpowers.
It should not replace Maud with a second framework-shaped abstraction unless the
 new surface is more honest and ergonomic than plain Maud.

For components specifically:

- the component struct is the semantic source of truth for props and slots
- slot-ness should live in the field type (`Slot<T>`, `Slot<Vec<T>>`), not in
  ad-hoc side metadata when avoidable
- Bon owns builder mechanics and typestate completion
- `maud-extensions` owns component semantics and final render assembly
- CSS and JS should feel component-owned and colocated
- builder `.render()` should produce the full component experience, not merely
  call `build()` and stop

## Preferred component surface

Today the preferred experimental authoring pattern is:

1. `#[derive(Component)]` on the struct
2. `Slot<Markup>` / `Slot<Vec<Markup>>` for slots
3. `#[mx(default)]` for the single default slot
4. `#[mx(each = item_name)]` for repeated slot item setters
5. `#[mx::component]` on the inherent impl block
6. `render! { ... }` for the component root
7. optional colocated `css! { ... }` and `js! { ... }` / `js!(once, { ... })`

Important: component authors should **not** need to manually wire an `impl
Render` that calls hidden hooks. The system should own the final render path.

## Means of achieving the philosophy

### Builder machinery

- use Bon for builder generation, typestate, completion, and internal setter
  wiring
- add semantic custom methods on top of Bon rather than reinventing builder
  state transitions
- let builder `.render()` be available only when Bon says the builder is
  complete

### Slot model

- single slots: `Slot<Markup>`
- repeated slots: `Slot<Vec<Markup>>`
- slots are declared by type, not by `#[mx(slot)]`
- `#[mx(slot)]` should be rejected with a migration hint toward `Slot<T>`
- multiple slot fields require exactly one `#[mx(default)]`

### Render protocol

- `render! { ... }` inside `#[mx::component]` impls is the owned render surface
- impl-local `css!` / `js!` blocks are turned into hidden hooks
- the generated render hook should assemble the root markup and inject those
  component-local CSS/JS blocks automatically
- the component system should validate the `render!` shape tightly enough that
  the injection point is honest and predictable

### CSS/JS story

- keep the existing standalone `css!` / `js!` macros as the small default story
- in component mode, colocated impl-block `css!` / `js!` are the premium story
- users should not have to manually emit `(Self::css())` or `(Self::js())` in
  component render bodies

## Current guardrails

- tuple/unit/unnamed-field structs are invalid component declarations
- legacy slot attrs are rejected
- repeated item setters require repeated slot storage
- duplicate `css!`, duplicate `js!`, and duplicate `render!` blocks in a
  component impl are rejected
- invalid impl-block JS mode forms are rejected

## Review checklist

When changing this component system, prefer solutions that keep these truths
intact:

- Bon remains the builder engine
- the slot declaration path stays type-driven
- component-local CSS/JS stay colocated and automatic
- component authors do not need extra boilerplate just to get the full render
  experience
- the public model stays smaller and more honest than a framework clone
