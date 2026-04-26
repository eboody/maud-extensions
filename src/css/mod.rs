// CSS semantic owner: coordinates css! parsing, validation, and emission.
mod dsl;
mod emit;
mod input;
mod source;
mod validate;

use proc_macro::TokenStream;

pub(crate) use input::MacroInput;

pub(crate) fn expand(input: MacroInput) -> TokenStream {
    match input {
        MacroInput::Inline(css_input) => TokenStream::from(emit::markup_tokens(css_input)),
        MacroInput::Named { helper_name, css } => emit::expand_named_helper(helper_name, css),
    }
}
