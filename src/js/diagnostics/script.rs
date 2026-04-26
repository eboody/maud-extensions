// JavaScript parser diagnostics.
use proc_macro2::Span;
use syn::Error;

use crate::js::validate::ScriptError;

use super::error;

pub(crate) fn invalid_script(span: Span, err: &ScriptError) -> Error {
    match err {
        ScriptError::ParserRejected {
            line,
            column,
            message,
        } => error(
            span,
            format!(
                "js! could not parse JavaScript (rendered JS line {line}, column {column}: {message})"
            ),
            "fix the JavaScript syntax near that rendered JS location",
        ),
    }
}
