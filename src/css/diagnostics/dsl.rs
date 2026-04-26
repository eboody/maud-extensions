// CSS helper-macro diagnostics.
use proc_macro2::Span;
use syn::Error;

use super::error;

pub(crate) fn raw_expects_exactly_one_string_literal(span: Span) -> Error {
    error(
        span,
        "raw! expects exactly one string literal argument",
        "write raw!(\"...\") when you need to inject literal CSS without DSL rewriting",
    )
}

pub(crate) fn macro_arguments_must_use_parentheses(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! expects macro arguments in parentheses"),
        format!("write {macro_name}!(...) instead of {macro_name}!{{...}} or {macro_name}![...]"),
    )
}

pub(crate) fn at_rule_requires_body(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! expects a prelude and trailing `{{ ... }}` body"),
        format!(
            "write {macro_name}!((condition), {{ ... }}) or {macro_name}!(\"condition\", {{ ... }})"
        ),
    )
}

pub(crate) fn at_rule_requires_prelude_then_body(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! expects `{macro_name}!(prelude, {{ ... }})`"),
        "place the prelude first, then a comma, then exactly one trailing brace-delimited CSS body",
    )
}

pub(crate) fn at_rule_accepts_only_prelude_and_body(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! only accepts a prelude and one body block"),
        "remove extra trailing arguments or fold that CSS into the single `{ ... }` body",
    )
}

pub(crate) fn at_rule_requires_non_empty_prelude(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! requires a non-empty prelude"),
        "pass the selector, query, or rule prelude before the body block",
    )
}

pub(crate) fn unit_requires_value(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! requires a value"),
        format!("write {macro_name}!(12) or {macro_name}!(1.5) with exactly one value"),
    )
}

pub(crate) fn unit_expects_exactly_one_value(span: Span, macro_name: &str) -> Error {
    error(
        span,
        format!("{macro_name}! expects exactly one value argument"),
        "remove extra comma-separated arguments so the helper expands to a single CSS value",
    )
}
