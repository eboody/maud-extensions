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
const COMPONENT_JS_HELPER_FN: &str =
    "__maud_extensions_component_requires_js_macro_in_scope_can_be_empty";
const COMPONENT_CSS_HELPER_FN: &str =
    "__maud_extensions_component_requires_css_macro_in_scope_can_be_empty";
const COMPONENT_JS_MODE_ATTR: &str = "data-mx-js-mode";
const COMPONENT_JS_RAN_ATTR: &str = "data-mx-js-ran";

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
            let css = tokens_to_css(tokens);
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
                // Stable, cheap hash. You can swap this for blake3 if you want.
                let mut h: u64 = 0xcbf29ce484222325; // FNV-1a offset
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

                // HTML id safe, short, deterministic.
                format!("{prefix}{h:016x}")
            }

            let __id = callsite_id(
                "mx-css-",
                file!(),
                line!(),
                column!(),
            );

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

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    expand_css_helper(tokens)
}

fn tokens_to_css(tokens: TokenStream2) -> String {
    let mut out = String::new();
    let mut prev_word = false;

    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ('(', ')'),
                    proc_macro2::Delimiter::Bracket => ('[', ']'),
                    proc_macro2::Delimiter::Brace => ('{', '}'),
                    proc_macro2::Delimiter::None => (' ', ' '),
                };
                let needs_space = prev_word
                    && matches!(
                        group.delimiter(),
                        proc_macro2::Delimiter::Brace | proc_macro2::Delimiter::None
                    );
                if needs_space {
                    out.push(' ');
                }
                if open != ' ' {
                    out.push(open);
                }
                out.push_str(&tokens_to_css(group.stream()));
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

fn expand_js_markup(js_input: JsInput) -> TokenStream {
    let (content_lit, js_string) = match js_input {
        JsInput::Literal(content) => {
            let js_string = content.value();
            (content, js_string)
        }
        JsInput::Tokens(tokens) => {
            let js = tokens_to_js(tokens);
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

#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let js_input = parse_macro_input!(input as JsInput);
    expand_js_helper(js_input)
}

#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let js_input = parse_macro_input!(input as JsInput);
    expand_js_markup(js_input)
}

#[proc_macro]
pub fn inline_css(input: TokenStream) -> TokenStream {
    let css_input = parse_macro_input!(input as CssInput);
    expand_css_markup(css_input)
}

fn component_syntax_error() -> syn::Error {
    syn::Error::new(
        Span::call_site(),
        "component! expects optional directives first (`@js-once` or `@js-always`) followed by exactly one top-level element with a body block, e.g. component! { @js-once article { ... } }",
    )
}

fn component_directive_error(message: &str) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}

fn is_punct(token: &TokenTree, ch: char) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == ch)
}

fn is_ident(token: &TokenTree, expected: &str) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident == expected)
}

fn parse_component_js_directive(tokens: &[TokenTree]) -> Result<(ComponentJsMode, usize)> {
    if tokens.len() < 4 {
        return Err(component_directive_error(
            "component! directive is incomplete. Use `@js-once` or `@js-always`.",
        ));
    }

    if !is_ident(&tokens[1], "js") || !is_punct(&tokens[2], '-') {
        return Err(component_directive_error(
            "unknown component! directive. Supported directives are `@js-once` and `@js-always`.",
        ));
    }

    let mode = if is_ident(&tokens[3], "once") {
        ComponentJsMode::Once
    } else if is_ident(&tokens[3], "always") {
        ComponentJsMode::Always
    } else {
        return Err(component_directive_error(
            "unknown component! directive. Supported directives are `@js-once` and `@js-always`.",
        ));
    };

    let mut consumed = 4usize;
    if matches!(tokens.get(consumed), Some(token) if is_punct(token, ';')) {
        consumed += 1;
    }
    Ok((mode, consumed))
}

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
        return component_syntax_error().to_compile_error().into();
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

    if tokens
        .iter()
        .any(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '@'))
    {
        return component_directive_error(
            "component! directives must appear before the root element.",
        )
        .to_compile_error()
        .into();
    }

    if !matches!(tokens.first(), Some(TokenTree::Ident(_))) {
        return component_syntax_error().to_compile_error().into();
    }

    let root_body_count = tokens
        .iter()
        .filter(|token| matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace))
        .count();

    if root_body_count != 1 {
        return component_syntax_error().to_compile_error().into();
    }

    let Some(TokenTree::Group(root_group)) = tokens.last() else {
        return component_syntax_error().to_compile_error().into();
    };
    if root_group.delimiter() != Delimiter::Brace {
        return component_syntax_error().to_compile_error().into();
    }

    let mut injected_body = root_group.stream();
    injected_body.extend(quote! { (#component_js_helper_ident()) (#component_css_helper_ident()) });
    let mut updated_group = Group::new(Delimiter::Brace, injected_body);
    updated_group.set_span(root_group.span());
    let last_index = tokens.len() - 1;
    tokens[last_index] = TokenTree::Group(updated_group);
    let js_mode_lit = LitStr::new(js_mode.as_str(), Span::call_site());
    tokens.splice(
        last_index..last_index,
        quote! {
            data-mx-component=""
            data-mx-js-mode=(#js_mode_lit)
        },
    );

    let root_tokens: TokenStream2 = tokens.into_iter().collect();
    let output = quote! {
        maud::html! {
            #root_tokens
        }
    };

    output.into()
}

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

#[proc_macro]
pub fn surreal_scope_inline(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as Nothing);
    let surreal_js = LitStr::new(SURREAL_JS_BUNDLE, Span::call_site());
    let css_scope_inline_js = LitStr::new(CSS_SCOPE_INLINE_JS_BUNDLE, Span::call_site());
    let output = quote! {
        maud::html! {
            script {
                (maud::PreEscaped(#surreal_js))
            }
            script {
                (maud::PreEscaped(#css_scope_inline_js))
            }
        }
    };

    TokenStream::from(output)
}

fn tokens_to_js(tokens: TokenStream2) -> String {
    let mut out = String::new();
    let mut prev_word = false;

    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ('(', ')'),
                    proc_macro2::Delimiter::Bracket => ('[', ']'),
                    proc_macro2::Delimiter::Brace => ('{', '}'),
                    proc_macro2::Delimiter::None => (' ', ' '),
                };
                let needs_space = prev_word
                    && matches!(
                        group.delimiter(),
                        proc_macro2::Delimiter::Brace | proc_macro2::Delimiter::None
                    );
                if needs_space {
                    out.push(' ');
                }
                if open != ' ' {
                    out.push(open);
                }
                out.push_str(&tokens_to_js(group.stream()));
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
    path: LitStr,
    family: LitStr,
    weight: Option<LitStr>,
    style: Option<LitStr>,
}

impl Parse for FontFace {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
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

#[proc_macro]
pub fn font_face(input: TokenStream) -> TokenStream {
    let font = parse_macro_input!(input as FontFace);

    let path = font.path;
    let family = font.family;
    let weight = font
        .weight
        .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));
    let style = font
        .style
        .unwrap_or_else(|| LitStr::new("normal", Span::call_site()));

    let expanded = quote! {
        {
            use base64::Engine;
            use base64::engine::general_purpose::STANDARD;
            use maud::PreEscaped;

            let font_bytes = include_bytes!(#path);
            let mut base64_string = String::new();

            STANDARD.encode_string(font_bytes, &mut base64_string);

            let path_str = #path;
            let format = if path_str.ends_with(".ttf") {
                "truetype"
            } else if path_str.ends_with(".otf") {
                "opentype"
            } else if path_str.ends_with(".woff") {
                "woff"
            } else if path_str.ends_with(".woff2") {
                "woff2"
            } else {
                "truetype"
            };

            let font_type = if path_str.ends_with(".woff2") {
                "woff2"
            } else if path_str.ends_with(".woff") {
                "woff"
            } else if path_str.ends_with(".otf") {
                "opentype"
            } else {
                "truetype"
            };

            let css = format!(
                "@font-face {{\n    font-family: '{}';\n    src: url('data:font/{};base64,{}') format('{}');\n    font-weight: {};\n    font-style: {};\n}}",
                #family,
                font_type,
                base64_string,
                format,
                #weight,
                #style
            );

            PreEscaped(css)
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn font_faces(input: TokenStream) -> TokenStream {
    let fonts = parse_macro_input!(input as FontFaceList);

    let font_faces = fonts.fonts.iter().map(|font| {
        let path = &font.path;
        let family = &font.family;
        let weight = font
            .weight
            .as_ref()
            .map_or_else(|| quote! { "normal" }, |w| quote! { #w });
        let style = font
            .style
            .as_ref()
            .map_or_else(|| quote! { "normal" }, |s| quote! { #s });

        quote! {
            {
                use maud_extensions::font_face;
                let face = font_face!(#path, #family, #weight, #style);
                css.push_str(&face.0);
            }
        }
    });

    let expanded = quote! {
        {
            use maud::PreEscaped;
            let mut css = String::new();

            #(#font_faces)*

            PreEscaped(css)
        }
    };

    expanded.into()
}
