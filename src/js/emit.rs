// JS emission: applies once semantics when needed and renders inline <script> markup.
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::LitStr;

use crate::js::{
    input::{Input, Mode},
    source, validate,
};
use crate::raw_text::sanitize_inline_script_source;

const RAN_ATTR: &str = "data-mx-js-ran";

pub(crate) fn markup_tokens(js_input: Input, mode: Mode) -> TokenStream2 {
    let js = match js_input {
        Input::Literal(content) => content.value(),
        Input::Tokens(tokens) => source::tokens_to_source(tokens),
    };

    let emitted = match mode {
        Mode::Always => js,
        Mode::Once => wrap_once(&js),
    };

    if let Err(message) = validate::script(&emitted) {
        return syn::Error::new(Span::call_site(), message).to_compile_error();
    }

    let sanitized = LitStr::new(&sanitize_inline_script_source(&emitted), Span::call_site());
    quote! {{
        maud::html! {
            script {
                (maud::PreEscaped(#sanitized))
            }
        }
    }}
}

pub(crate) fn expand_named_helper(helper_name: Ident, mode: Mode, js_input: Input) -> TokenStream {
    let js_markup = markup_tokens(js_input, mode);
    TokenStream::from(quote! {
        fn #helper_name() -> maud::Markup {
            #js_markup
        }
    })
}

fn wrap_once(js: &str) -> String {
    format!(
        "const __mx_script = document.currentScript;\n\
         const __mx_root = __mx_script && __mx_script.parentElement;\n\
         let __mx_should_run = true;\n\
         if (__mx_root) {{\n\
         if (__mx_root.hasAttribute(\"{RAN_ATTR}\")) {{\n\
         __mx_should_run = false;\n\
         }}\n\
         }}\n\
         if (__mx_should_run) {{\n\
         {}\n\
         if (__mx_root) {{\n\
         __mx_root.setAttribute(\"{RAN_ATTR}\", \"\");\n\
         }}\n\
         }}",
        js
    )
}
