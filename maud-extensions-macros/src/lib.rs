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
pub fn surreal_scope_inline(_input: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "`surreal_scope_inline!()` is no longer part of the preferred story; use `mx::Init::new().surrealjs().scoped_css().build()` or `mx::Init::all()` instead",
    )
    .to_compile_error()
    .into()
}

/// Emits bundled Signals runtime and adapter.
#[proc_macro]
pub fn signals_inline(_input: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "`signals_inline!()` is no longer part of the preferred story; use `mx::Init::new().signals().build()` or `mx::Init::all()` instead",
    )
    .to_compile_error()
    .into()
}

/// Emits the full runtime bundle: Surreal, css-scope-inline, Signals core, and adapter.
#[proc_macro]
pub fn surreal_scope_signals_inline(_input: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "`surreal_scope_signals_inline!()` is no longer part of the preferred story; use `mx::Init::all()` instead",
    )
    .to_compile_error()
    .into()
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

/// Removed component impl macro.
#[cfg(feature = "components")]
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "`#[mx::component]` is no longer part of the component story; use explicit `fn css() -> Markup`, `fn js() -> Markup`, and a normal `impl Render` instead",
    )
    .to_compile_error()
    .into()
}

/// Removed render block macro from the component story.
#[cfg(feature = "components")]
#[proc_macro]
pub fn render(_input: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "`render!` is no longer part of the component story; write a normal `impl Render` and place `(Self::css())` / `(Self::js())` explicitly in the markup",
    )
    .to_compile_error()
    .into()
}
