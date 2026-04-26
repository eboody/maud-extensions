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

extern crate self as maud_extensions;

mod slot;

pub use maud_extensions_macros::{css, js};
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
