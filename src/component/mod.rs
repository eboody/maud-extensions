// Experimental component authoring owner: derive parsing, semantic modeling,
// and expansion for builder-centric Maud components.
mod diagnostic;
mod expand;
mod input;
mod model;

use proc_macro::TokenStream;

pub(crate) use input::Input;

pub(crate) fn expand(input: Input) -> TokenStream {
    expand::derive(model::Component::from_input(input))
}
