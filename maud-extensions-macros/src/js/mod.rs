// JS semantic owner: coordinates js! parsing, validation, and emission.
mod diagnostics;
mod emit;
mod input;
mod source;
mod validate;

use proc_macro::TokenStream;

pub(crate) use input::MacroInput;

pub(crate) fn expand(input: MacroInput) -> TokenStream {
    match input {
        MacroInput::Inline { mode, js } => TokenStream::from(emit::markup_tokens(js, mode)),
        MacroInput::Named {
            helper_name,
            mode,
            js,
        } => emit::expand_named_helper(helper_name, mode, js),
    }
}
