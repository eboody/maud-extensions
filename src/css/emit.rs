// CSS emission: validates final source and renders the inline <style> markup.
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::LitStr;

use crate::css::{diagnostics, input::Input, source, validate};
use crate::raw_text::sanitize_inline_style_source;

pub(crate) fn markup_tokens(css_input: Input) -> TokenStream2 {
    let input_span = css_input.span();
    let css = match css_input {
        Input::Literal(content) => content.value(),
        Input::Tokens(tokens) => match source::tokens_to_source(tokens) {
            Ok(css) => css,
            Err(err) => return err.to_compile_error(),
        },
    };

    if let Err(err) = validate::stylesheet(&css) {
        return diagnostics::invalid_stylesheet(input_span, &err).to_compile_error();
    }

    let content_lit = LitStr::new(&sanitize_inline_style_source(&css), Span::call_site());
    quote! {{
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
    }}
}

pub(crate) fn expand_named_helper(helper_name: Ident, css_input: Input) -> TokenStream {
    let css_markup = markup_tokens(css_input);
    TokenStream::from(quote! {
        fn #helper_name() -> maud::Markup {
            #css_markup
        }
    })
}
