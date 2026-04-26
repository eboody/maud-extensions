// Stylesheet validation diagnostics.
use proc_macro2::Span;
use syn::Error;

use crate::css::validate::StylesheetError;

use crate::internal_diagnostics::error_with_help;

pub(crate) fn invalid_stylesheet(span: Span, err: &StylesheetError) -> Error {
    match err {
        StylesheetError::UnterminatedComment => error_with_help(
            span,
            "css! found an unterminated comment",
            "close every `/*` comment with a matching `*/` before the end of the css! body",
        ),
        StylesheetError::UnmatchedClosing { delimiter } => error_with_help(
            span,
            format!("css! found an unmatched closing `{delimiter}` in the stylesheet"),
            "remove the extra closing delimiter or add the matching opening delimiter earlier in the stylesheet",
        ),
        StylesheetError::UnterminatedString => error_with_help(
            span,
            "css! found an unterminated string literal",
            "close the quoted CSS string before the end of the css! body",
        ),
        StylesheetError::UnclosedDelimiter { delimiter } => error_with_help(
            span,
            format!("css! found an unclosed `{delimiter}` delimiter in the stylesheet"),
            format!(
                "close every `{delimiter}` with its matching delimiter before the end of the css! body"
            ),
        ),
        StylesheetError::ParserRejectedTokens { location, message } => error_with_help(
            span,
            format!(
                "css! could not parse CSS tokens after rendering the stylesheet (rendered CSS line {}, column {}: {message})",
                location.line + 1,
                location.column,
            ),
            "check for malformed CSS token syntax near that rendered CSS location, or wrap verbatim text in raw!(\"...\") when the DSL should not rewrite it",
        ),
    }
}
