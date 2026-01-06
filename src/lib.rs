use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};
use syn::{
    LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

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

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let css_input = parse_macro_input!(input as CssInput);
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

        pub fn callsite_id(prefix: &str, file: &str, line: u32, col: u32) -> String {
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
            style {
                (maud::PreEscaped(#content_lit))
            }
        }
    };

    TokenStream::from(output)
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
                _ => return Err("css! could not parse CSS tokens".to_string()),
            },
        }
    }
}

#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let js_input = parse_macro_input!(input as JsInput);
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

#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    let output = quote! {
        fn js() -> maud::Markup {
            ::maud_extensions::js! { #tokens }
        }
    };

    TokenStream::from(output)
}

#[proc_macro]
pub fn inline_css(input: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = input.into();
    let output = quote! {
        fn css() -> maud::Markup {
            ::maud_extensions::css! { #tokens }
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
    let fm = cm.new_source_file(FileName::Custom("inline.js".to_string()), js.to_string());
    let input = StringInput::from(&*fm);
    let mut parser = Parser::new(Syntax::Es(EsConfig::default()), input, None);
    match parser.parse_script() {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("js! could not parse JavaScript: {err:#?}")),
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
