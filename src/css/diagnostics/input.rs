// css! input-shape diagnostics.
use proc_macro2::Span;
use syn::Error;

use super::error;

pub(crate) fn helper_name_must_be_identifier(span: Span) -> Error {
    error(
        span,
        "css! named helper names must be Rust identifiers, not string literals",
        "write css!(card_styles, { ... }) instead of css!(\"card-styles\", { ... })",
    )
}

pub(crate) fn unexpected_trailing_tokens_after_named_helper(span: Span) -> Error {
    error(
        span,
        "unexpected trailing tokens after named css! helper",
        "css! named helpers only accept `css!(helper_name, { ... })` or `css!(helper_name, \"...\")`",
    )
}
