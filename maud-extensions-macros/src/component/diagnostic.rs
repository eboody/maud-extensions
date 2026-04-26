// Component derive diagnostics.
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

use crate::component::model::Component;

pub(crate) fn unsupported_in_v1(
    component: &Component,
    field_name: &str,
    feature: &str,
) -> TokenStream {
    let name = &component.name;
    syn::Error::new(
        name.span(),
        format!(
            "#[derive(Component)] v1 does not support {feature} yet; field `{field_name}` in component `{name}` requires a later slice"
        ),
    )
    .to_compile_error()
}

pub(crate) fn from_model_error(err: Error) -> TokenStream {
    let rewritten = match err.to_string().as_str() {
        "component-only-structs" => Error::new(
            err.span(),
            "#[derive(Component)] currently only supports structs",
        ),
        "component-only-named-fields" => Error::new(
            err.span(),
            "#[derive(Component)] currently only supports structs with named fields",
        ),
        _ => err,
    };

    rewritten.to_compile_error()
}

pub(crate) fn tokens(ts: TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! { #ts })
}
