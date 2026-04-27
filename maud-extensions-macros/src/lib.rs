#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
// Public proc-macro surface: canonical css!/js!/Component entrypoints live here.
//! Local CSS and JS emitters for Maud.
//!
//! The reset story for this branch is intentionally small:
//! - write plain `html!`
//! - emit local CSS with `css!`
//! - emit local JS with `js!`
//! - optionally opt into experimental component authoring with `Component`
//!
//! `maud-extensions` should feel like tiny Maud superpowers, not a framework.

#[cfg(feature = "components")]
mod component;
mod css;
mod internal_diagnostics;
mod js;
mod raw_text;
mod runtime;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Emits a `<style>` tag or defines a named local CSS helper.
///
/// Supported forms:
/// - `css! { ... }`
/// - `css!(name, { ... })`
///
/// ```ignore
/// use maud::html;
/// use maud_extensions::{css, js};
///
/// fn view() -> maud::Markup {
///     css!(responsive_css, {
///         media!("(min-width: 48rem)", {
///             me { padding: rem!(2); }
///         })
///     });
///
///     html! {
///         article.card {
///             "Hello"
///             (css! {
///                 me { color: red; }
///             })
///             (responsive_css())
///             (js! {
///                 me().class_add("ready");
///             })
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as css::MacroInput);
    css::expand(input)
}

/// Emits a `<script>` tag or defines a named local JS helper.
///
/// Supported forms:
/// - `js! { ... }`
/// - `js!(once, { ... })`
/// - `js!(name, { ... })`
/// - `js!(name, once, { ... })`
///
/// ```ignore
/// use maud::html;
/// use maud_extensions::js;
///
/// fn view() -> maud::Markup {
///     html! {
///         div {
///             (js!(once, {
///                 me().class_add("ready");
///             }))
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as js::MacroInput);
    js::expand(input)
}

/// Emits bundled Surreal and css-scope-inline runtimes.
#[proc_macro]
pub fn surreal_scope_inline(input: TokenStream) -> TokenStream {
    runtime::expand_runtime_bundle(input, runtime::Bundle::SurrealScope)
}

/// Emits bundled Signals runtime and adapter.
#[proc_macro]
pub fn signals_inline(input: TokenStream) -> TokenStream {
    runtime::expand_runtime_bundle(input, runtime::Bundle::Signals)
}

/// Emits the full runtime bundle: Surreal, css-scope-inline, Signals core, and adapter.
#[proc_macro]
pub fn surreal_scope_signals_inline(input: TokenStream) -> TokenStream {
    runtime::expand_runtime_bundle(input, runtime::Bundle::SurrealScopeSignals)
}

/// Experimental opt-in derive for builder-centric Maud component authoring.
///
/// This derive is gated behind the `components` crate feature and is only a
/// partial surface right now. The current v1 slice is Bon-backed and supports
/// prop builders only; slot-specific ergonomics are still under construction.
///
/// Generated component code references `::maud_extensions::bon`. The public
/// runtime crate re-exports Bon so downstream users only need `maud-extensions`.
/// Repeated slot support currently relies on Bon's experimental `overwritable`
/// feature through that re-exported dependency.
#[cfg(feature = "components")]
#[proc_macro_derive(Component, attributes(mx))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as component::Input);
    component::expand(input)
}
