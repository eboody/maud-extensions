// Component derive diagnostics.
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn not_yet_implemented(name: &Ident) -> TokenStream {
    syn::Error::new(
        name.span(),
        "#[derive(Component)] is not implemented yet; this experimental surface is scaffolded but not available yet",
    )
    .to_compile_error()
}

pub(crate) fn only_structs(name: &Ident) -> TokenStream {
    syn::Error::new(
        name.span(),
        "#[derive(Component)] currently only supports structs",
    )
    .to_compile_error()
}

pub(crate) fn tokens(ts: TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! { #ts })
}
