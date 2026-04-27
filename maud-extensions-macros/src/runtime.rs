// Bundled runtime emitters for browser-side helper scripts.
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::LitStr;

use crate::raw_text::sanitize_inline_script_source;

pub(crate) enum Bundle {
    SurrealScope,
    Signals,
    SurrealScopeSignals,
}

pub(crate) fn expand_runtime_bundle(input: TokenStream, bundle: Bundle) -> TokenStream {
    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "runtime emitter macros do not accept arguments")
            .to_compile_error()
            .into();
    }

    let scripts = match bundle {
        Bundle::SurrealScope => vec![
            runtime_script(include_str!("../assets/surreal.js")),
            runtime_script(include_str!("../assets/css-scope-inline.js")),
        ],
        Bundle::Signals => vec![
            runtime_script(include_str!("../assets/signals-core.min.js")),
            runtime_script(include_str!("../assets/signals-adapter.js")),
        ],
        Bundle::SurrealScopeSignals => vec![
            runtime_script(include_str!("../assets/surreal.js")),
            runtime_script(include_str!("../assets/css-scope-inline.js")),
            runtime_script(include_str!("../assets/signals-core.min.js")),
            runtime_script(include_str!("../assets/signals-adapter.js")),
        ],
    };

    TokenStream::from(quote! {{
        ::maud::html! {
            @for __script in [#(#scripts),*] {
                script { (::maud::PreEscaped(__script)) }
            }
        }
    }})
}

fn runtime_script(source: &str) -> LitStr {
    LitStr::new(&sanitize_inline_script_source(source), Span::call_site())
}
