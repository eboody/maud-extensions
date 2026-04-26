// Shared compile-time diagnostic formatting helpers.
use proc_macro2::Span;
use syn::Error;

pub(crate) fn error_with_help(
    span: Span,
    summary: impl AsRef<str>,
    help: impl AsRef<str>,
) -> Error {
    Error::new(
        span,
        format!("{}\nhelp: {}", summary.as_ref(), help.as_ref()),
    )
}
