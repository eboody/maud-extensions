// Semantic component model built from the user's struct declaration.
use syn::Ident;

use crate::component::input::Input;

pub(crate) struct Component {
    pub(crate) name: Ident,
    pub(crate) is_struct: bool,
}

impl Component {
    pub(crate) fn from_input(input: Input) -> Self {
        Self {
            name: input.ident,
            is_struct: matches!(input.data, syn::Data::Struct(_)),
        }
    }
}
