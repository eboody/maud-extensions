#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
//! Tiny Maud superpowers.
//!
//! This crate is the public runtime/support surface:
//! - reexports proc macros like [`css!`] and [`js!`]
//! - optionally reexports the experimental [`Component`] derive
//! - optionally reexports the experimental [`component`] impl macro
//! - provides runtime slot wrapper types like [`Slot`] and [`Slots`]
//! - reexports `bon` so generated component code can depend only on
//!   `maud-extensions`
//!
//! Recommended dependency spelling:
//!
//! ```toml
//! [dependencies]
//! mx = { package = "maud-extensions", version = "0.6.2", features = ["components"] }
//! ```
//!
//! # Experimental components
//!
//! The component system is currently opt-in behind the `components` feature.
//! The macro that turns impl-local component rendering and asset blocks on is:
//!
//! ```ignore
//! #[mx::component]
//! impl Card {
//!     render! { ... }
//!     css! { ... }
//!     js!(once, { ... });
//! }
//! ```
//!
//! That impl macro is what makes `render!`, `css!`, and `js!` part of the
//! component render pipeline.
//!
//! The preferred authoring pattern is:
//!
//! - `#[derive(Component)]` on the struct
//! - `Slot<maud::Markup>` / `Slot<Vec<maud::Markup>>` for slot fields
//! - `#[mx(default)]` on the single default slot
//! - `#[mx::component]` on the inherent impl block
//! - `render! { ... }` for the component root
//! - optional colocated `css! { ... }` and `js!(once, { ... })` blocks
//!
//! The builder `.render()` path uses the hidden [`ComponentRender`] hook and
//! currently auto-injects impl-local CSS and JS into the rendered root.
//!
//! ```ignore
//! use maud::Markup;
//! use mx::{Component, Slot};
//!
//! #[derive(Component)]
//! struct Card {
//!     title: String,
//!     header: Slot<Markup>,
//!     #[mx(default)]
//!     body: Slot<Markup>,
//!     footer: Slot<Markup>,
//!     #[mx(each = action)]
//!     actions: Slot<Vec<Markup>>,
//! }
//!
//! #[mx::component]
//! impl Card {
//!     css! {
//!         me {
//!             padding: 1rem;
//!             border: 1px solid #ddd;
//!         }
//!     }
//!
//!     js!(once, {
//!         me().class_add("ready");
//!     });
//!
//!     render! {
//!         article.card {
//!             header class="header" { (self.header) }
//!             h2 { (self.title) }
//!             div.body { (self.body) }
//!             footer class="footer" { (self.footer) }
//!             div.actions { (self.actions) }
//!         }
//!     }
//! }
//! ```
//!
//! The broader browser-side runtime pieces this layers on top of today include:
//!
//! - Surreal: <https://github.com/gnat/surreal>
//! - css-scope-inline: <https://github.com/gnat/css-scope-inline>
//! - Preact Signals: <https://github.com/preactjs/signals>
//!
//! To bootstrap those browser-side pieces in a page, emit one of the bundled
//! runtime macros in `<head>`:
//!
//! ```ignore
//! use maud::html;
//! use mx::surreal_scope_signals_inline;
//!
//! fn page() -> maud::Markup {
//!     html! {
//!         head { (surreal_scope_signals_inline!()) }
//!         body { /* page body */ }
//!     }
//! }
//! ```

extern crate self as maud_extensions;

mod slot;

pub use maud_extensions_macros::{css, js, signals_inline, surreal_scope_inline, surreal_scope_signals_inline};
pub use slot::{Slot, Slots};

#[cfg(feature = "components")]
pub use maud_extensions_macros::Component;
#[cfg(feature = "components")]
pub use maud_extensions_macros::component;

#[doc(hidden)]
pub use bon;

/// Hidden render hook used by the component builder render path.
#[doc(hidden)]
pub trait ComponentRender {
    /// Renders the fully assembled component, including any impl-local facets.
    fn __mx_render(&self) -> maud::Markup;
}
