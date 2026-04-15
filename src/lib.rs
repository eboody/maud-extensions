#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
//! Proc macros for Maud views with component-scoped helpers and runtime assets.
//!
//! Supported workflows:
//! - `js!`, `css!`, and `component!` for file-scoped components
//! - `inline_js!`, `inline_css!`, `js_file!`, and `css_file!` for direct asset injection
//! - `surreal_scope_inline!()` for the bundled `surreal.js` and `css-scope-inline.js`
//! - `signals_inline!()` and `surreal_scope_signals_inline!()` for bundled Signals helpers
//! - `font_face!` and `font_faces!` for embedding font files as data URLs
//!
//! Support policy:
//! - MSRV: Rust 1.85
//! - Supported Maud version: 0.27
//!
//! Important limits:
//! - `component!` accepts exactly one top-level Maud element with a body block. It doesn't accept
//!   control-flow roots or every possible Maud token pattern.
//! - `inline_js!` parses the emitted JavaScript with SWC before generating markup.
//! - `inline_css!` performs a lightweight syntax check before forwarding the stylesheet as written.
//! - Signals support stays JS-first: markup provides anchors, while `js!` owns signals and DOM
//!   binding.
//! - Slot helpers live in the companion `maud-extensions-runtime` crate.

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};
use syn::{
    Data, DeriveInput, Expr, Fields, GenericParam, Generics, LitStr, Result, Token, Type, TypePath,
    parse::{Nothing, Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

const SURREAL_JS_BUNDLE: &str = include_str!("../assets/surreal.js");
const CSS_SCOPE_INLINE_JS_BUNDLE: &str = include_str!("../assets/css-scope-inline.js");
const SIGNALS_CORE_JS_BUNDLE: &str = include_str!("../assets/signals-core.min.js");
const SIGNALS_ADAPTER_JS_BUNDLE: &str = include_str!("../assets/signals-adapter.js");
const COMPONENT_JS_HELPER_FN: &str =
    "__maud_extensions_component_requires_js_macro_in_scope_can_be_empty";
const COMPONENT_CSS_HELPER_FN: &str =
    "__maud_extensions_component_requires_css_macro_in_scope_can_be_empty";
const COMPONENT_JS_MODE_ATTR: &str = "data-mx-js-mode";
const COMPONENT_JS_RAN_ATTR: &str = "data-mx-js-ran";
const COMPONENT_SYNTAX_ERROR: &str = "component! expects optional directives first (`@js-once` or `@js-always`) followed by exactly one top-level element with a body block, e.g. component! { @js-once article { ... } }";

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentJsMode {
    Always,
    Once,
}

impl ComponentJsMode {
    fn as_str(self) -> &'static str {
        match self {
            ComponentJsMode::Always => "always",
            ComponentJsMode::Once => "once",
        }
    }
}

enum JsInput {
    Literal(LitStr),
    Tokens(TokenStream2),
}

impl Parse for JsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            let content: LitStr = input.parse()?;
            Ok(JsInput::Literal(content))
        } else {
            let tokens: TokenStream2 = input.parse()?;
            Ok(JsInput::Tokens(tokens))
        }
    }
}

enum CssInput {
    Literal(LitStr),
    Tokens(TokenStream2),
}

impl Parse for CssInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            let content: LitStr = input.parse()?;
            Ok(CssInput::Literal(content))
        } else {
            let tokens: TokenStream2 = input.parse()?;
            Ok(CssInput::Tokens(tokens))
        }
    }
}

fn expand_css_markup(css_input: CssInput) -> TokenStream {
    let content_lit = match css_input {
        CssInput::Literal(content) => content,
        CssInput::Tokens(tokens) => {
            let css = tokens_to_source(tokens);
            if let Err(message) = validate_css(&css) {
                return syn::Error::new(Span::call_site(), message)
                    .to_compile_error()
                    .into();
            }
            LitStr::new(&css, Span::call_site())
        }
    };

    let output = quote! {
        {
            fn callsite_id(prefix: &str, file: &str, line: u32, col: u32) -> String {
                let mut h: u64 = 0xcbf29ce484222325;
                for b in file.as_bytes() {
                    h ^= *b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
                for b in line.to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
                for b in col.to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }

                format!("{prefix}{h:016x}")
            }

            let __id = callsite_id("mx-css-", file!(), line!(), column!());

            maud::html! {
                style data-mx-css-id=(__id) {
                    (maud::PreEscaped(#content_lit))
                }
            }
        }
    };

    TokenStream::from(output)
}

fn expand_css_helper(tokens: TokenStream2) -> TokenStream {
    let component_css_helper_ident = Ident::new(COMPONENT_CSS_HELPER_FN, Span::call_site());
    let output = quote! {
        fn css() -> maud::Markup {
            ::maud_extensions::inline_css! { #tokens }
        }

        #[doc(hidden)]
        fn #component_css_helper_ident() -> maud::Markup {
            css()
        }
    };

    TokenStream::from(output)
}

/// Generates a local `fn css() -> maud::Markup` helper for `component!`.
///
/// The macro accepts either a string literal or CSS-like tokens. Token input is flattened into a
/// stylesheet string and checked for basic CSS syntax before it is emitted.
///
/// ```rust
/// use maud_extensions::{component, css, js};
///
/// fn view() -> maud::Markup {
///     js! {}
///     let markup = component! {
///         div class="card" {
///             "Hello"
///         }
///     };
///     css! {
///         me { color: red; }
///     }
///     markup
/// }
/// ```
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    expand_css_helper(tokens)
}

fn tokens_to_source(tokens: TokenStream2) -> String {
    let mut out = String::new();
    let mut prev_word = false;

    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::None => (' ', ' '),
                };
                let needs_space =
                    prev_word && matches!(group.delimiter(), Delimiter::Brace | Delimiter::None);
                if needs_space {
                    out.push(' ');
                }
                if open != ' ' {
                    out.push(open);
                }
                out.push_str(&tokens_to_source(group.stream()));
                if close != ' ' {
                    out.push(close);
                }
                prev_word = false;
            }
            TokenTree::Ident(ident) => {
                if prev_word {
                    out.push(' ');
                }
                out.push_str(&ident.to_string());
                prev_word = true;
            }
            TokenTree::Literal(literal) => {
                if prev_word {
                    out.push(' ');
                }
                out.push_str(&literal.to_string());
                prev_word = true;
            }
            TokenTree::Punct(punct) => {
                out.push(punct.as_char());
                prev_word = false;
            }
        }
    }

    out
}

fn validate_css(css: &str) -> core::result::Result<(), String> {
    let mut input = cssparser::ParserInput::new(css);
    let mut parser = cssparser::Parser::new(&mut input);
    loop {
        match parser.next_including_whitespace_and_comments() {
            Ok(_) => {}
            Err(err) => match err.kind {
                cssparser::BasicParseErrorKind::EndOfInput => return Ok(()),
                _ => return Err("inline_css! could not parse CSS tokens".to_string()),
            },
        }
    }
}

fn emit_script_bundles(bundles: impl IntoIterator<Item = &'static str>) -> TokenStream {
    let bundles: Vec<LitStr> = bundles
        .into_iter()
        .map(|bundle| LitStr::new(bundle, Span::call_site()))
        .collect();

    quote! {
        maud::html! {
            #(
                script {
                    (maud::PreEscaped(#bundles))
                }
            )*
        }
    }
    .into()
}

fn expand_js_markup(js_input: JsInput) -> TokenStream {
    let (content_lit, js_string) = match js_input {
        JsInput::Literal(content) => {
            let js_string = content.value();
            (content, js_string)
        }
        JsInput::Tokens(tokens) => {
            let js = tokens_to_source(tokens);
            (LitStr::new(&js, Span::call_site()), js)
        }
    };
    if let Err(message) = validate_js(&js_string) {
        return syn::Error::new(Span::call_site(), message)
            .to_compile_error()
            .into();
    }

    let output = quote! {
        maud::html! {
            script {
                (maud::PreEscaped(#content_lit))
            }
        }
    };

    TokenStream::from(output)
}

fn expand_js_helper(js_input: JsInput) -> TokenStream {
    let component_js_helper_ident = Ident::new(COMPONENT_JS_HELPER_FN, Span::call_site());
    let js_mode_attr = COMPONENT_JS_MODE_ATTR;
    let js_ran_attr = COMPONENT_JS_RAN_ATTR;
    let js_markup = match js_input {
        JsInput::Literal(content) => {
            let wrapped = format!(
                "const __mx_script = document.currentScript;\n\
                 const __mx_root = __mx_script && __mx_script.parentElement;\n\
                 const __mx_mode = __mx_root ? __mx_root.getAttribute(\"{js_mode_attr}\") : null;\n\
                 let __mx_should_run = true;\n\
                 if (__mx_mode === \"once\" && __mx_root) {{\n\
                 if (__mx_root.hasAttribute(\"{js_ran_attr}\")) {{\n\
                 __mx_should_run = false;\n\
                 }} else {{\n\
                 __mx_root.setAttribute(\"{js_ran_attr}\", \"\");\n\
                 }}\n\
                 }}\n\
                 if (__mx_should_run) {{\n\
                 {}\n\
                 }}",
                content.value()
            );
            let wrapped_lit = LitStr::new(&wrapped, Span::call_site());
            quote! {
                ::maud_extensions::inline_js!(#wrapped_lit)
            }
        }
        JsInput::Tokens(tokens) => {
            let js_mode_attr = LitStr::new(js_mode_attr, Span::call_site());
            let js_ran_attr = LitStr::new(js_ran_attr, Span::call_site());
            quote! {
                ::maud_extensions::inline_js! {
                    const __mx_script = document.currentScript;
                    const __mx_root = __mx_script && __mx_script.parentElement;
                    const __mx_mode = __mx_root ? __mx_root.getAttribute(#js_mode_attr) : null;

                    let __mx_should_run = true;
                    if (__mx_mode === "once" && __mx_root) {
                        if (__mx_root.hasAttribute(#js_ran_attr)) {
                            __mx_should_run = false;
                        } else {
                            __mx_root.setAttribute(#js_ran_attr, "");
                        }
                    }

                    if (__mx_should_run) {
                        #tokens
                    }
                }
            }
        }
    };

    let output = quote! {
        fn js() -> maud::Markup {
            #js_markup
        }

        #[doc(hidden)]
        fn #component_js_helper_ident() -> maud::Markup {
            js()
        }
    };

    TokenStream::from(output)
}

/// Generates a local `fn js() -> maud::Markup` helper for `component!`.
///
/// The macro accepts either a string literal or JavaScript-like tokens. The generated helper is
/// wrapped so `component!` can honor `@js-once` and `@js-always`.
///
/// ```rust
/// use maud_extensions::{component, css, js};
///
/// fn view() -> maud::Markup {
///     js! {
///         me().class_add("ready");
///     }
///     let markup = component! {
///         div class="card" {
///             "Hello"
///         }
///     };
///     css! {}
///     markup
/// }
/// ```
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let js_input = parse_macro_input!(input as JsInput);
    expand_js_helper(js_input)
}

/// Emits a `<script>` tag directly from a JavaScript string literal or token block.
///
/// The JavaScript is parsed with SWC before the markup is generated.
///
/// ```rust
/// use maud_extensions::inline_js;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         (inline_js! {
///             console.log("ready");
///         })
///     }
/// }
/// ```
#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let js_input = parse_macro_input!(input as JsInput);
    expand_js_markup(js_input)
}

/// Emits a `<style>` tag directly from a CSS string literal or token block.
///
/// The CSS is checked for basic syntax errors before the markup is generated.
///
/// ```rust
/// use maud_extensions::inline_css;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         (inline_css! {
///             .card { display: block; }
///         })
///     }
/// }
/// ```
#[proc_macro]
pub fn inline_css(input: TokenStream) -> TokenStream {
    let css_input = parse_macro_input!(input as CssInput);
    expand_css_markup(css_input)
}

fn component_syntax_error(span: Span) -> syn::Error {
    syn::Error::new(span, COMPONENT_SYNTAX_ERROR)
}

fn component_directive_error(span: Span, message: &str) -> syn::Error {
    syn::Error::new(span, message)
}

fn is_punct(token: &TokenTree, ch: char) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == ch)
}

fn is_ident(token: &TokenTree, expected: &str) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident == expected)
}

fn token_span(token: Option<&TokenTree>) -> Span {
    token.map(TokenTree::span).unwrap_or_else(Span::call_site)
}

fn parse_component_js_directive(tokens: &[TokenTree]) -> Result<(ComponentJsMode, usize)> {
    if tokens.len() < 4 {
        return Err(component_directive_error(
            token_span(tokens.first()),
            "component! directive is incomplete. Use `@js-once` or `@js-always`.",
        ));
    }

    if !is_ident(&tokens[1], "js") || !is_punct(&tokens[2], '-') {
        return Err(component_directive_error(
            tokens[1].span(),
            "unknown component! directive. Supported directives are `@js-once` and `@js-always`.",
        ));
    }

    let mode = if is_ident(&tokens[3], "once") {
        ComponentJsMode::Once
    } else if is_ident(&tokens[3], "always") {
        ComponentJsMode::Always
    } else {
        return Err(component_directive_error(
            tokens[3].span(),
            "unknown component! directive. Supported directives are `@js-once` and `@js-always`.",
        ));
    };

    let mut consumed = 4usize;
    if matches!(tokens.get(consumed), Some(token) if is_punct(token, ';')) {
        consumed += 1;
    }
    Ok((mode, consumed))
}

fn find_component_body_index(tokens: &[TokenTree]) -> Result<usize> {
    if tokens.is_empty() {
        return Err(component_syntax_error(Span::call_site()));
    }
    if !matches!(tokens.first(), Some(TokenTree::Ident(_))) {
        return Err(component_syntax_error(token_span(tokens.first())));
    }

    if let Some(token) = tokens
        .iter()
        .find(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '@'))
    {
        return Err(component_directive_error(
            token.span(),
            "component! directives must appear before the root element.",
        ));
    }

    let Some(body_index) = tokens.iter().position(
        |token| matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace),
    ) else {
        return Err(component_syntax_error(token_span(tokens.last())));
    };

    let trailing = tokens
        .iter()
        .enumerate()
        .skip(body_index + 1)
        .find(|(_, token)| !matches!(token, TokenTree::Punct(punct) if punct.as_char() == ';'));
    if let Some((_, token)) = trailing {
        return Err(component_syntax_error(token.span()));
    }

    Ok(body_index)
}

/// Wraps a single top-level Maud element and injects the local `js!` and `css!` helpers inside
/// that root element.
///
/// `component!` performs compile-time shape checks over the token stream it observes. It accepts
/// one top-level element with a body block plus an optional `@js-once` or `@js-always` directive.
///
/// ```rust
/// use maud_extensions::{component, css, js};
///
/// fn view() -> maud::Markup {
///     js! {
///         me().class_add("ready");
///     }
///     let markup = component! {
///         @js-once
///         article class="card" {
///             p { "Hello" }
///         }
///     };
///     css! {
///         me { border: 1px solid #ddd; }
///     }
///     markup
/// }
/// ```
#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let component_js_helper_ident = Ident::new(COMPONENT_JS_HELPER_FN, Span::call_site());
    let component_css_helper_ident = Ident::new(COMPONENT_CSS_HELPER_FN, Span::call_site());
    let mut tokens: Vec<TokenTree> = TokenStream2::from(input).into_iter().collect();

    while matches!(
        tokens.last(),
        Some(TokenTree::Punct(punct)) if punct.as_char() == ';'
    ) {
        tokens.pop();
    }

    if tokens.is_empty() {
        return component_syntax_error(Span::call_site())
            .to_compile_error()
            .into();
    }

    let mut js_mode = ComponentJsMode::Always;
    let mut seen_mode_directive = false;
    let mut consumed = 0usize;

    while matches!(tokens.get(consumed), Some(token) if is_punct(token, '@')) {
        let (mode, directive_len) = match parse_component_js_directive(&tokens[consumed..]) {
            Ok(parsed) => parsed,
            Err(err) => return err.to_compile_error().into(),
        };

        if seen_mode_directive {
            return component_directive_error(
                tokens[consumed].span(),
                "component! accepts at most one JS mode directive (`@js-once` or `@js-always`).",
            )
            .to_compile_error()
            .into();
        }

        js_mode = mode;
        seen_mode_directive = true;
        consumed += directive_len;
    }

    if consumed > 0 {
        tokens.drain(0..consumed);
    }

    let body_index = match find_component_body_index(&tokens) {
        Ok(index) => index,
        Err(err) => return err.to_compile_error().into(),
    };

    let Some(TokenTree::Group(root_group)) = tokens.get(body_index) else {
        return component_syntax_error(token_span(tokens.last()))
            .to_compile_error()
            .into();
    };

    let mut injected_body = root_group.stream();
    injected_body.extend(quote! { (#component_js_helper_ident()) (#component_css_helper_ident()) });
    let mut updated_group = Group::new(Delimiter::Brace, injected_body);
    updated_group.set_span(root_group.span());
    tokens[body_index] = TokenTree::Group(updated_group);

    let js_mode_lit = LitStr::new(js_mode.as_str(), Span::call_site());
    tokens.splice(
        body_index..body_index,
        quote! {
            data-mx-component=""
            data-mx-js-mode=(#js_mode_lit)
        },
    );

    let root_tokens: TokenStream2 = tokens.into_iter().collect();
    quote! {
        maud::html! {
            #root_tokens
        }
    }
    .into()
}

/// Emits a `<script>` tag from a file path accepted by `include_str!`.
///
/// ```rust
/// use maud_extensions::js_file;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         (js_file!(concat!(
///             env!("CARGO_MANIFEST_DIR"),
///             "/tests/fixtures/runtime.js"
///         )))
///     }
/// }
/// ```
#[proc_macro]
pub fn js_file(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as Expr);
    let output = quote! {
        maud::html! {
            script {
                (maud::PreEscaped(include_str!(#path)))
            }
        }
    };

    TokenStream::from(output)
}

/// Emits a `<style>` tag from a file path accepted by `include_str!`.
///
/// ```rust
/// use maud_extensions::css_file;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         (css_file!(concat!(
///             env!("CARGO_MANIFEST_DIR"),
///             "/tests/fixtures/runtime.css"
///         )))
///     }
/// }
/// ```
#[proc_macro]
pub fn css_file(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as Expr);
    let output = quote! {
        maud::html! {
            style {
                (maud::PreEscaped(include_str!(#path)))
            }
        }
    };

    TokenStream::from(output)
}

/// Emits the bundled `surreal.js` and `css-scope-inline.js` runtime helpers.
///
/// ```rust
/// use maud_extensions::surreal_scope_inline;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         head {
///             (surreal_scope_inline!())
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn surreal_scope_inline(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as Nothing);
    emit_script_bundles([SURREAL_JS_BUNDLE, CSS_SCOPE_INLINE_JS_BUNDLE])
}

/// Emits the bundled Signals core runtime plus the Maud adapter helpers.
///
/// This macro installs the `window.mx` namespace and the binder helpers used by the
/// `surreal_scope_signals_inline!()` workflow.
///
/// ```rust
/// use maud_extensions::signals_inline;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         head {
///             (signals_inline!())
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn signals_inline(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as Nothing);
    emit_script_bundles([SIGNALS_CORE_JS_BUNDLE, SIGNALS_ADAPTER_JS_BUNDLE])
}

/// Emits the bundled `surreal.js`, `css-scope-inline.js`, Signals core, and Maud Signals adapter.
///
/// This is the supported runtime include when a page uses `component!`, `js!`, and the Signals
/// DOM binders together.
///
/// ```rust
/// use maud_extensions::surreal_scope_signals_inline;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         head {
///             (surreal_scope_signals_inline!())
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn surreal_scope_signals_inline(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as Nothing);
    emit_script_bundles([
        SURREAL_JS_BUNDLE,
        CSS_SCOPE_INLINE_JS_BUNDLE,
        SIGNALS_CORE_JS_BUNDLE,
        SIGNALS_ADAPTER_JS_BUNDLE,
    ])
}

fn validate_js(js: &str) -> core::result::Result<(), String> {
    let cm = SourceMap::default();
    let fm = cm.new_source_file(
        FileName::Custom("inline.js".to_string()).into(),
        js.to_string(),
    );
    let input = StringInput::from(&*fm);
    let mut parser = Parser::new(Syntax::Es(EsSyntax::default()), input, None);
    match parser.parse_script() {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("inline_js! could not parse JavaScript: {err:#?}")),
    }
}

struct FontFace {
    path: Expr,
    family: LitStr,
    weight: Option<LitStr>,
    style: Option<LitStr>,
}

impl Parse for FontFace {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let family: LitStr = input.parse()?;

        let weight = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.peek(LitStr) {
                Some(input.parse()?)
            } else {
                None
            }
        } else {
            None
        };

        let style = if weight.is_some() && input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.peek(LitStr) {
                Some(input.parse()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(FontFace {
            path,
            family,
            weight,
            style,
        })
    }
}

struct FontFaceList {
    fonts: Punctuated<FontFace, Token![;]>,
}

impl Parse for FontFaceList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fonts = Punctuated::parse_terminated(input)?;
        Ok(FontFaceList { fonts })
    }
}

fn expand_font_face_css(
    path: &Expr,
    family: &LitStr,
    weight: &LitStr,
    style: &LitStr,
) -> TokenStream2 {
    quote! {{
        fn __mx_encode_base64(bytes: &[u8]) -> String {
            const TABLE: &[u8; 64] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

            let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
            let mut chunks = bytes.chunks_exact(3);
            for chunk in &mut chunks {
                let combined =
                    ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
                out.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
                out.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
                out.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
                out.push(TABLE[(combined & 0x3f) as usize] as char);
            }

            match chunks.remainder() {
                [only] => {
                    let combined = (*only as u32) << 16;
                    out.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
                    out.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
                    out.push('=');
                    out.push('=');
                }
                [first, second] => {
                    let combined = ((*first as u32) << 16) | ((*second as u32) << 8);
                    out.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
                    out.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
                    out.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
                    out.push('=');
                }
                [] => {}
                _ => unreachable!("chunks_exact(3) only leaves 0, 1, or 2 trailing bytes"),
            }

            out
        }

        static __MX_FONT_FACE_CSS: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();

        __MX_FONT_FACE_CSS
            .get_or_init(|| {
                let __mx_bytes = include_bytes!(#path);
                let __mx_path = (#path).to_ascii_lowercase();
                let (__mx_font_type, __mx_format) = if __mx_path.ends_with(".woff2") {
                    ("woff2", "woff2")
                } else if __mx_path.ends_with(".woff") {
                    ("woff", "woff")
                } else if __mx_path.ends_with(".otf") {
                    ("opentype", "opentype")
                } else {
                    ("truetype", "truetype")
                };
                let __mx_base64 = __mx_encode_base64(__mx_bytes);
                format!(
                    "@font-face {{\n    font-family: '{}';\n    src: url('data:font/{};base64,{}') format('{}');\n    font-weight: {};\n    font-style: {};\n}}",
                    #family,
                    __mx_font_type,
                    __mx_base64,
                    __mx_format,
                    #weight,
                    #style
                )
            })
            .clone()
    }}
}

/// Embeds a font file as a single `@font-face` block.
///
/// The path expression must be accepted by `include_bytes!`, for example a string literal or
/// `concat!(env!("CARGO_MANIFEST_DIR"), "/path/to/font.woff2")`.
///
/// ```rust
/// use maud_extensions::font_face;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         style {
///             (font_face!(
///                 concat!(
///                     env!("CARGO_MANIFEST_DIR"),
///                     "/examples/assets/demo-font.woff2"
///                 ),
///                 "Demo Sans"
///             ))
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn font_face(input: TokenStream) -> TokenStream {
    let font = parse_macro_input!(input as FontFace);

    let weight = font
        .weight
        .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));
    let style = font
        .style
        .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));
    let css = expand_font_face_css(&font.path, &font.family, &weight, &style);

    quote! {{
        maud::PreEscaped(#css)
    }}
    .into()
}

/// Embeds multiple font files as adjacent `@font-face` blocks.
///
/// ```rust
/// use maud_extensions::font_faces;
///
/// fn view() -> maud::Markup {
///     maud::html! {
///         style {
///             (font_faces!(
///                 concat!(
///                     env!("CARGO_MANIFEST_DIR"),
///                     "/examples/assets/demo-font.woff2"
///                 ), "Demo Sans";
///                 concat!(
///                     env!("CARGO_MANIFEST_DIR"),
///                     "/examples/assets/demo-font-bold.woff2"
///                 ), "Demo Sans", "700", "normal"
///             ))
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn font_faces(input: TokenStream) -> TokenStream {
    let fonts = parse_macro_input!(input as FontFaceList);

    let font_faces = fonts.fonts.iter().map(|font| {
        let weight = font
            .weight
            .as_ref()
            .cloned()
            .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));
        let style = font
            .style
            .as_ref()
            .cloned()
            .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));
        let css = expand_font_face_css(&font.path, &font.family, &weight, &style);

        quote! {
            css.push_str(&#css);
        }
    });

    quote! {{
        let mut css = String::new();
        #(#font_faces)*
        maud::PreEscaped(css)
    }}
    .into()
}

#[derive(Clone)]
enum BuilderFieldKind {
    Required,
    Optional { inner: Type },
    Repeated { inner: Type },
    Defaulted,
}

#[derive(Clone, Default)]
struct SlotAttr {
    is_slot: bool,
    is_default: bool,
}

#[derive(Clone, Default)]
struct BuilderAttr {
    use_default: bool,
    each_method: Option<Ident>,
}

#[derive(Clone)]
enum BuilderInputMode {
    Direct(Type),
    RenderToMarkup,
}

#[derive(Clone)]
struct BuilderField {
    ident: Ident,
    ty: Type,
    kind: BuilderFieldKind,
    slot: SlotAttr,
    builder: BuilderAttr,
    setter_input: BuilderInputMode,
    repeated_item_input: Option<BuilderInputMode>,
    state_ident: Option<Ident>,
}

/// Derives a typed builder for a named component struct.
///
/// Required fields are plain fields unless they use `Option<T>`, `Vec<T>`, or `#[builder(default)]`.
/// The builder exposes one setter per field plus optional repeated-item setters from
/// `#[builder(each = "item_name")]`. Fields written as `Markup`, `maud::Markup`, or
/// `::maud::Markup` accept any `maud::Render` value in their generated setters.
///
/// Slot metadata can be declared with `#[slot]` and `#[slot(default)]` so the component contract
/// is explicit before higher-level composition sugar exists.
///
/// ```rust
/// use maud::{Markup, Render, html};
/// use maud_extensions::ComponentBuilder;
///
/// #[derive(ComponentBuilder)]
/// struct Card<'a> {
///     title: &'a str,
///     #[slot(optional)]
///     header: Option<Markup>,
///     #[slot(default)]
///     body: Markup,
/// }
///
/// impl<'a> Render for Card<'a> {
///     fn render(&self) -> Markup {
///         html! {
///             article {
///                 @if let Some(header) = &self.header {
///                     header { (header) }
///                 }
///                 main { (self.body) }
///             }
///         }
///     }
/// }
///
/// fn view() -> Markup {
///     Card::new()
///         .title("Status")
///         .header(html! { h2 { "Live" } })
///         .body(html! { p { "All systems green" } })
///         .build()
///         .render()
/// }
/// ```
#[proc_macro_derive(ComponentBuilder, attributes(builder, slot))]
pub fn component_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_component_builder(input)
}

fn expand_component_builder(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let vis = input.vis;
    let generics = input.generics;

    let Data::Struct(data_struct) = input.data else {
        return syn::Error::new(
            ident.span(),
            "ComponentBuilder only supports structs with named fields.",
        )
        .to_compile_error()
        .into();
    };

    let Fields::Named(fields_named) = data_struct.fields else {
        return syn::Error::new(
            ident.span(),
            "ComponentBuilder only supports structs with named fields.",
        )
        .to_compile_error()
        .into();
    };

    let parsed_fields = match fields_named
        .named
        .iter()
        .enumerate()
        .map(|(index, field)| parse_builder_field(index, field))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    if let Err(err) = validate_builder_fields(&parsed_fields) {
        return err.to_compile_error().into();
    }

    let builder_ident = format_ident!("{ident}Builder");
    let existing_args = generic_args_from_generics(&generics);

    let required_fields: Vec<&BuilderField> = parsed_fields
        .iter()
        .filter(|field| matches!(field.kind, BuilderFieldKind::Required))
        .collect();

    let mut builder_generics = generics.clone();
    for field in &required_fields {
        let state_ident = field
            .state_ident
            .as_ref()
            .expect("required fields always carry a state ident");
        builder_generics
            .params
            .push(parse_quote!(const #state_ident: bool));
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (_builder_impl_generics, _builder_ty_generics, builder_where_clause) =
        builder_generics.split_for_impl();

    let new_builder_ty = builder_type_tokens(
        &builder_ident,
        &existing_args,
        required_fields.iter().map(|_| quote!(false)).collect(),
    );

    let builder_struct_fields = parsed_fields.iter().map(|field| {
        let ident = &field.ident;
        let storage_ty = builder_storage_ty(field);
        quote! { #ident: #storage_ty }
    });

    let builder_init_fields = parsed_fields.iter().map(|field| {
        let ident = &field.ident;
        let init = builder_init_expr(field);
        quote! { #ident: #init }
    });

    let component_new_impl = quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #[must_use]
            pub fn new() -> #new_builder_ty {
                #builder_ident {
                    #(#builder_init_fields),*
                }
            }

            #[must_use]
            pub fn builder() -> #new_builder_ty {
                Self::new()
            }
        }
    };

    let setters = parsed_fields
        .iter()
        .map(|field| {
            let method = expand_builder_field_setter(
                &builder_ident,
                &existing_args,
                &builder_generics,
                &parsed_fields,
                &required_fields,
                field,
            );
            let each = expand_builder_each_setter(
                &builder_ident,
                &existing_args,
                &builder_generics,
                &parsed_fields,
                &required_fields,
                field,
            );

            quote! {
                #method
                #each
            }
        })
        .collect::<Vec<_>>();

    let build_impl = expand_builder_build_impl(
        &ident,
        &builder_ident,
        &generics,
        &existing_args,
        &parsed_fields,
        &required_fields,
    );

    let output = quote! {
        #vis struct #builder_ident #builder_generics #builder_where_clause {
            #(#builder_struct_fields),*
        }

        #component_new_impl
        #(#setters)*
        #build_impl
    };

    output.into()
}

fn parse_builder_field(field_index: usize, field: &syn::Field) -> syn::Result<BuilderField> {
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| syn::Error::new(field.span(), "ComponentBuilder requires named fields."))?;

    let slot = parse_slot_attr(&field.attrs)?;
    let builder = parse_builder_attr(&field.attrs)?;
    let kind = classify_builder_field(&field.ty, builder.use_default);
    let setter_input = match &kind {
        BuilderFieldKind::Repeated { .. } => BuilderInputMode::Direct(field.ty.clone()),
        _ => setter_input_mode(&field.ty, &kind),
    };
    let repeated_item_input = repeated_item_input_mode(&kind);
    let state_ident = matches!(kind, BuilderFieldKind::Required)
        .then(|| format_ident!("__MAUD_EXTENSIONS_REQUIRED_FIELD_{field_index}_SET"));

    Ok(BuilderField {
        ident,
        ty: field.ty.clone(),
        kind,
        slot,
        builder,
        setter_input,
        repeated_item_input,
        state_ident,
    })
}

fn parse_slot_attr(attrs: &[syn::Attribute]) -> syn::Result<SlotAttr> {
    let mut slot = SlotAttr::default();

    for attr in attrs {
        if !attr.path().is_ident("slot") {
            continue;
        }

        slot.is_slot = true;
        if matches!(&attr.meta, syn::Meta::Path(_)) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                slot.is_default = true;
                return Ok(());
            }

            if meta.path.is_ident("optional") {
                return Ok(());
            }

            Err(meta.error(
                "unsupported slot attribute. Supported forms are `#[slot]` and `#[slot(default)]`.",
            ))
        })?;
    }

    Ok(slot)
}

fn parse_builder_attr(attrs: &[syn::Attribute]) -> syn::Result<BuilderAttr> {
    let mut builder = BuilderAttr::default();

    for attr in attrs {
        if !attr.path().is_ident("builder") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                builder.use_default = true;
                return Ok(());
            }

            if meta.path.is_ident("each") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                builder.each_method = Some(Ident::new(&lit.value(), lit.span()));
                return Ok(());
            }

            Err(meta.error(
                "unsupported builder attribute. Supported forms are `#[builder(default)]` and `#[builder(each = \"item\")]`.",
            ))
        })?;
    }

    Ok(builder)
}

fn classify_builder_field(ty: &Type, use_default: bool) -> BuilderFieldKind {
    if use_default {
        return BuilderFieldKind::Defaulted;
    }

    if let Some(inner) = option_inner_ty(ty) {
        return BuilderFieldKind::Optional { inner };
    }

    if let Some(inner) = vec_inner_ty(ty) {
        return BuilderFieldKind::Repeated { inner };
    }

    BuilderFieldKind::Required
}

fn validate_builder_fields(fields: &[BuilderField]) -> syn::Result<()> {
    let default_slots = fields.iter().filter(|field| field.slot.is_default).count();
    if default_slots > 1 {
        let duplicate = fields
            .iter()
            .find(|field| field.slot.is_default)
            .expect("count verified");
        return Err(syn::Error::new(
            duplicate.ident.span(),
            "ComponentBuilder allows at most one `#[slot(default)]` field.",
        ));
    }

    for field in fields {
        if field.builder.each_method.is_some()
            && !matches!(field.kind, BuilderFieldKind::Repeated { .. })
        {
            return Err(syn::Error::new(
                field.ident.span(),
                "`#[builder(each = \"...\")]` only applies to `Vec<T>` fields.",
            ));
        }

        if let Some(each) = &field.builder.each_method {
            if each == &field.ident {
                return Err(syn::Error::new(
                    each.span(),
                    "`#[builder(each = \"...\")]` must use a method name different from the field name.",
                ));
            }
        }
    }

    let mut method_names = std::collections::BTreeSet::new();
    method_names.insert("build".to_string());

    for field in fields {
        let field_method = field.ident.to_string();
        if !method_names.insert(field_method.clone()) {
            return Err(syn::Error::new(
                field.ident.span(),
                format!("duplicate generated builder method `{field_method}`."),
            ));
        }

        if let Some(each) = &field.builder.each_method {
            let each_method = each.to_string();
            if !method_names.insert(each_method.clone()) {
                return Err(syn::Error::new(
                    each.span(),
                    format!("duplicate generated builder method `{each_method}`."),
                ));
            }
        }
    }

    Ok(())
}

fn option_inner_ty(ty: &Type) -> Option<Type> {
    generic_inner_ty(
        ty,
        &[
            &["Option"],
            &["std", "option", "Option"],
            &["core", "option", "Option"],
        ],
    )
}

fn vec_inner_ty(ty: &Type) -> Option<Type> {
    generic_inner_ty(
        ty,
        &[&["Vec"], &["std", "vec", "Vec"], &["alloc", "vec", "Vec"]],
    )
}

fn generic_inner_ty(ty: &Type, accepted_paths: &[&[&str]]) -> Option<Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };

    if !path_matches_any(path, accepted_paths) {
        return None;
    }

    let segment = path.segments.last()?;
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    if args.args.len() != 1 {
        return None;
    }

    let syn::GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };

    Some(inner.clone())
}

fn is_markup_ty(ty: &Type) -> bool {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return false;
    };

    path_matches_any(path, &[&["Markup"], &["maud", "Markup"]])
}

fn path_matches_any(path: &syn::Path, accepted_paths: &[&[&str]]) -> bool {
    accepted_paths
        .iter()
        .any(|segments| path_matches_segments(path, segments))
}

fn path_matches_segments(path: &syn::Path, expected_segments: &[&str]) -> bool {
    if path.segments.len() != expected_segments.len() {
        return false;
    }

    path.segments
        .iter()
        .zip(expected_segments.iter())
        .all(|(segment, expected)| segment.ident == expected)
}

fn builder_storage_ty(field: &BuilderField) -> TokenStream2 {
    match field.kind {
        BuilderFieldKind::Required => {
            let ty = &field.ty;
            quote!(::core::option::Option<#ty>)
        }
        _ => {
            let ty = &field.ty;
            quote!(#ty)
        }
    }
}

fn builder_init_expr(field: &BuilderField) -> TokenStream2 {
    match field.kind {
        BuilderFieldKind::Required => quote!(::core::option::Option::None),
        BuilderFieldKind::Optional { .. } => quote!(::core::option::Option::None),
        BuilderFieldKind::Repeated { .. } => quote!(::std::vec::Vec::new()),
        BuilderFieldKind::Defaulted => quote!(::core::default::Default::default()),
    }
}

fn setter_input_mode(ty: &Type, kind: &BuilderFieldKind) -> BuilderInputMode {
    match kind {
        BuilderFieldKind::Required | BuilderFieldKind::Defaulted => {
            if is_markup_ty(ty) {
                BuilderInputMode::RenderToMarkup
            } else {
                BuilderInputMode::Direct(ty.clone())
            }
        }
        BuilderFieldKind::Optional { inner } => {
            if is_markup_ty(inner) {
                BuilderInputMode::RenderToMarkup
            } else {
                BuilderInputMode::Direct(inner.clone())
            }
        }
        BuilderFieldKind::Repeated { .. } => {
            unreachable!("repeated fields use repeated_item_input_mode")
        }
    }
}

fn repeated_item_input_mode(kind: &BuilderFieldKind) -> Option<BuilderInputMode> {
    let BuilderFieldKind::Repeated { inner } = kind else {
        return None;
    };

    Some(if is_markup_ty(inner) {
        BuilderInputMode::RenderToMarkup
    } else {
        BuilderInputMode::Direct(inner.clone())
    })
}

fn generic_args_from_generics(generics: &Generics) -> Vec<TokenStream2> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote!(#ident)
            }
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote!(#lifetime)
            }
            GenericParam::Const(param) => {
                let ident = &param.ident;
                quote!(#ident)
            }
        })
        .collect()
}

fn builder_type_tokens(
    builder_ident: &Ident,
    existing_args: &[TokenStream2],
    state_args: Vec<TokenStream2>,
) -> TokenStream2 {
    let mut all_args = existing_args.to_vec();
    all_args.extend(state_args);

    if all_args.is_empty() {
        quote!(#builder_ident)
    } else {
        quote!(#builder_ident < #(#all_args),* >)
    }
}

fn expand_builder_field_setter(
    builder_ident: &Ident,
    existing_args: &[TokenStream2],
    builder_generics: &Generics,
    fields: &[BuilderField],
    required_fields: &[&BuilderField],
    field: &BuilderField,
) -> TokenStream2 {
    let (impl_generics, _ty_generics, where_clause) = builder_generics.split_for_impl();
    let method_ident = &field.ident;
    let current_state_args = required_fields
        .iter()
        .map(|required| {
            let state_ident = required
                .state_ident
                .as_ref()
                .expect("required field state ident");
            quote!(#state_ident)
        })
        .collect::<Vec<_>>();

    let return_state_args = required_fields
        .iter()
        .map(|required| {
            if required.ident == field.ident {
                quote!(true)
            } else {
                let state_ident = required
                    .state_ident
                    .as_ref()
                    .expect("required field state ident");
                quote!(#state_ident)
            }
        })
        .collect::<Vec<_>>();

    let current_ty = builder_type_tokens(builder_ident, existing_args, current_state_args);
    let return_ty = builder_type_tokens(builder_ident, existing_args, return_state_args);
    let rebuild_fields = fields.iter().map(|other| {
        let ident = &other.ident;
        if ident == &field.ident {
            let value_expr = setter_value_expr(field);
            quote!(#ident: #value_expr)
        } else {
            quote!(#ident: self.#ident)
        }
    });

    let (arg_tokens, setter_prelude) = setter_arg_tokens(field);

    quote! {
        impl #impl_generics #current_ty #where_clause {
            #[must_use]
            pub fn #method_ident(self, #arg_tokens) -> #return_ty {
                #setter_prelude
                #builder_ident {
                    #(#rebuild_fields),*
                }
            }
        }
    }
}

fn expand_builder_each_setter(
    builder_ident: &Ident,
    existing_args: &[TokenStream2],
    builder_generics: &Generics,
    fields: &[BuilderField],
    required_fields: &[&BuilderField],
    field: &BuilderField,
) -> TokenStream2 {
    let Some(each_ident) = &field.builder.each_method else {
        return TokenStream2::new();
    };

    let (impl_generics, _ty_generics, where_clause) = builder_generics.split_for_impl();
    let current_state_args = required_fields
        .iter()
        .map(|required| {
            let state_ident = required
                .state_ident
                .as_ref()
                .expect("required field state ident");
            quote!(#state_ident)
        })
        .collect::<Vec<_>>();

    let current_ty = builder_type_tokens(builder_ident, existing_args, current_state_args);
    let repeated_field_ident = &field.ident;
    let rebuild_fields = fields.iter().map(|other| {
        let ident = &other.ident;
        if ident == repeated_field_ident {
            quote!(#ident: #repeated_field_ident)
        } else {
            quote!(#ident: self.#ident)
        }
    });

    let (arg_tokens, push_expr) = each_setter_arg_tokens(field);

    quote! {
        impl #impl_generics #current_ty #where_clause {
            #[must_use]
            pub fn #each_ident(self, #arg_tokens) -> Self {
                let mut #repeated_field_ident = self.#repeated_field_ident;
                #push_expr
                #builder_ident {
                    #(#rebuild_fields),*
                }
            }
        }
    }
}

fn setter_arg_tokens(field: &BuilderField) -> (TokenStream2, TokenStream2) {
    match &field.kind {
        BuilderFieldKind::Repeated { .. } => match field
            .repeated_item_input
            .as_ref()
            .expect("repeated fields always expose item input")
        {
            BuilderInputMode::Direct(inner) => (
                quote!(values: impl ::core::iter::IntoIterator<Item = #inner>),
                quote!(),
            ),
            BuilderInputMode::RenderToMarkup => (
                quote!(values: impl ::core::iter::IntoIterator<Item = impl ::maud::Render>),
                quote!(),
            ),
        },
        _ => match &field.setter_input {
            BuilderInputMode::Direct(ty) => (quote!(value: #ty), quote!()),
            BuilderInputMode::RenderToMarkup => (quote!(value: impl ::maud::Render), quote!()),
        },
    }
}

fn setter_value_expr(field: &BuilderField) -> TokenStream2 {
    match &field.kind {
        BuilderFieldKind::Required => match &field.setter_input {
            BuilderInputMode::Direct(_) => quote!(::core::option::Option::Some(value)),
            BuilderInputMode::RenderToMarkup => {
                quote!(::core::option::Option::Some(::maud::Render::render(&value)))
            }
        },
        BuilderFieldKind::Optional { .. } => match &field.setter_input {
            BuilderInputMode::Direct(_) => quote!(::core::option::Option::Some(value)),
            BuilderInputMode::RenderToMarkup => {
                quote!(::core::option::Option::Some(::maud::Render::render(&value)))
            }
        },
        BuilderFieldKind::Repeated { .. } => match field
            .repeated_item_input
            .as_ref()
            .expect("repeated fields always expose item input")
        {
            BuilderInputMode::Direct(_) => quote!(values.into_iter().collect()),
            BuilderInputMode::RenderToMarkup => {
                quote!(
                    values
                        .into_iter()
                        .map(|value| ::maud::Render::render(&value))
                        .collect()
                )
            }
        },
        BuilderFieldKind::Defaulted => match &field.setter_input {
            BuilderInputMode::Direct(_) => quote!(value),
            BuilderInputMode::RenderToMarkup => quote!(::maud::Render::render(&value)),
        },
    }
}

fn each_setter_arg_tokens(field: &BuilderField) -> (TokenStream2, TokenStream2) {
    let repeated_field_ident = &field.ident;
    match field
        .repeated_item_input
        .as_ref()
        .expect("each setters only exist for repeated fields")
    {
        BuilderInputMode::Direct(inner) => (
            quote!(value: #inner),
            quote!(#repeated_field_ident.push(value);),
        ),
        BuilderInputMode::RenderToMarkup => (
            quote!(value: impl ::maud::Render),
            quote!(#repeated_field_ident.push(::maud::Render::render(&value));),
        ),
    }
}

fn expand_builder_build_impl(
    component_ident: &Ident,
    builder_ident: &Ident,
    generics: &Generics,
    existing_args: &[TokenStream2],
    fields: &[BuilderField],
    required_fields: &[&BuilderField],
) -> TokenStream2 {
    let component_ty = if existing_args.is_empty() {
        quote!(#component_ident)
    } else {
        quote!(#component_ident < #(#existing_args),* >)
    };

    let complete_builder_ty = builder_type_tokens(
        builder_ident,
        existing_args,
        required_fields.iter().map(|_| quote!(true)).collect(),
    );

    let build_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        match field.kind {
            BuilderFieldKind::Required => {
                let field_name = ident.to_string();
                quote! {
                    #ident: #ident.expect(concat!(
                        "ComponentBuilder state bug: missing required field `",
                        #field_name,
                        "` at build time."
                    ))
                }
            }
            _ => quote!(#ident: #ident),
        }
    });

    let destructure_fields = fields.iter().map(|field| &field.ident);
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #complete_builder_ty #where_clause {
            #[must_use]
            pub fn build(self) -> #component_ty {
                let Self { #(#destructure_fields),* } = self;
                #component_ident {
                    #(#build_fields),*
                }
            }
        }

        impl #impl_generics ::core::convert::From<#complete_builder_ty> for #component_ty #where_clause {
            fn from(builder: #complete_builder_ty) -> Self {
                builder.build()
            }
        }
    }
}
