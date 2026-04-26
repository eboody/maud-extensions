// js! input-shape diagnostics.
use proc_macro2::Span;
use syn::Error;

use super::error;

pub(crate) fn helper_name_must_be_identifier(span: Span) -> Error {
    error(
        span,
        "js! named helper names must be Rust identifiers, not string literals",
        "write js!(card_script, { ... }) instead of js!(\"card-script\", { ... })",
    )
}

pub(crate) fn unexpected_trailing_tokens_after_body(span: Span) -> Error {
    error(
        span,
        "unexpected trailing tokens after js! body",
        "js! inline forms only accept `js!({ ... })`, `js!(\"...\")`, `js!(once, { ... })`, or `js!(once, \"...\")`",
    )
}

pub(crate) fn named_helper_mode_must_be_once(span: Span) -> Error {
    error(
        span,
        "js! named helper mode must be `once`",
        "use `js!(helper_name, once, { ... })` for once-only helpers, or omit the mode entirely",
    )
}

pub(crate) fn unexpected_trailing_tokens_after_named_helper(span: Span) -> Error {
    error(
        span,
        "unexpected trailing tokens after named js! helper",
        "js! named helpers only accept `js!(helper_name, { ... })`, `js!(helper_name, \"...\")`, `js!(helper_name, once, { ... })`, or `js!(helper_name, once, \"...\")`",
    )
}
