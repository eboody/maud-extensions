// Component derive expansion orchestration.
use proc_macro::TokenStream;

use crate::component::{diagnostic, model::Component};

pub(crate) fn derive(component: Component) -> TokenStream {
    if !component.is_struct {
        return diagnostic::tokens(diagnostic::only_structs(&component.name));
    }

    diagnostic::tokens(diagnostic::not_yet_implemented(&component.name))
}
