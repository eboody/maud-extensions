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
use quote::quote;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};
use syn::{
    Expr, LitStr, Result, Token,
    parse::{Nothing, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
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
