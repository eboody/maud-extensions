// Component derive expansion orchestration.
use proc_macro::TokenStream;

use crate::component::{diagnostic, model::Component};

pub(crate) fn derive(component: syn::Result<Component>) -> TokenStream {
    match component {
        Ok(component) => diagnostic::tokens(diagnostic::not_yet_implemented(&component)),
        Err(err) => diagnostic::tokens(diagnostic::from_model_error(err)),
    }
}
