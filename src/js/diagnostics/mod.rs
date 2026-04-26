// JS compile-time diagnostics: shared wording and focused constructors.
mod input;
mod script;

use proc_macro2::Span;
use syn::Error;

pub(crate) use input::*;
pub(crate) use script::*;

fn error(span: Span, summary: impl AsRef<str>, help: impl AsRef<str>) -> Error {
    Error::new(
        span,
        format!("{}\nhelp: {}", summary.as_ref(), help.as_ref()),
    )
}
