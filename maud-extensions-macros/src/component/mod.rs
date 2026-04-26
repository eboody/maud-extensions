// Experimental component authoring owner: derive parsing, semantic modeling,
// and expansion for builder-centric Maud components.
mod attrs;
mod diagnostic;
mod expand;
mod field;
mod impl_block;
mod input;
mod model;

use proc_macro::TokenStream;
use syn::ItemImpl;

pub(crate) use input::Input;

pub(crate) fn expand(input: Input) -> TokenStream {
    expand::derive(model::Component::from_input(input))
}

pub(crate) fn expand_impl(input: ItemImpl) -> TokenStream {
    impl_block::expand(input)
}
