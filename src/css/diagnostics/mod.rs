// CSS compile-time diagnostics: shared wording and focused constructors.
mod dsl;
mod input;
mod stylesheet;

use proc_macro2::Span;
use syn::Error;

pub(crate) use dsl::*;
pub(crate) use input::*;
pub(crate) use stylesheet::*;

fn error(span: Span, summary: impl AsRef<str>, help: impl AsRef<str>) -> Error {
    Error::new(
        span,
        format!("{}\nhelp: {}", summary.as_ref(), help.as_ref()),
    )
}
