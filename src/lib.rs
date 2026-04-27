#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
//! Tiny Maud superpowers.
//!
//! This crate is the public runtime/support surface:
//! - reexports proc macros like [`css!`] and [`js!`]
//! - optionally reexports the experimental [`Component`] derive
//! - provides runtime slot wrapper types like [`Slot`] and [`Slots`]
//! - reexports `bon` so generated component code can depend only on
//!   `maud-extensions`
//!
//! Recommended dependency spelling:
//!
//! ```toml
//! [dependencies]
//! mx = { package = "maud-extensions", version = "0.6.6", features = ["components"] }
//! ```
//!
//! # Experimental components
//!
//! The component system is currently opt-in behind the `components` feature.
//! The preferred authoring pattern is:
//!
//! - `#[derive(Component)]` on the struct
//! - `Slot<maud::Markup>` / `Slot<Vec<maud::Markup>>` for slot fields
//! - `#[mx(default)]` on the single default slot
//! - reserve `#[mx(default)]` for slot selection only; use Rust `Default` or
//!   `Option<T>` for non-slot defaults
//! - ordinary inherent helpers like `fn css() -> Markup` and
//!   `fn js() -> Markup`
//! - ordinary `impl Render` with explicit `(Self::css())` / `(Self::js())`
//!   emission where desired
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
//! impl Card {
//!     fn css() -> Markup {
//!         mx::css! {
//!             me {
//!                 padding: 1rem;
//!                 border: 1px solid #ddd;
//!             }
//!         }
//!     }
//!
//!     fn js() -> Markup {
//!         mx::js!(once, {
//!             me().class_add("ready");
//!         })
//!     }
//! }
//!
//! impl maud::Render for Card {
//!     fn render(&self) -> Markup {
//!         maud::html! {
//!             article.card {
//!                 (Self::css())
//!                 (Self::js())
//!                 header class="header" { (self.header) }
//!                 h2 { (self.title) }
//!                 div.body { (self.body) }
//!                 footer class="footer" { (self.footer) }
//!                 div.actions { (self.actions) }
//!             }
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
//! To bootstrap those browser-side pieces in a page, prefer [`Init`] in
//! `<head>`:
//!
//! ```ignore
//! use maud::html;
//! use mx::Init;
//!
//! fn page() -> maud::Markup {
//!     html! {
//!         head { (Init::all()) }
//!         body { /* page body */ }
//!     }
//! }
//! ```

extern crate self as maud_extensions;

mod init;
mod slot;

pub use maud_extensions_macros::{
    css, js, signals_inline, surreal_scope_inline, surreal_scope_signals_inline,
};
pub use init::Init;
pub use slot::{Slot, Slots};

#[cfg(feature = "components")]
pub use maud_extensions_macros::Component;

#[doc(hidden)]
pub use bon;
