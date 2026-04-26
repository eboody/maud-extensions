// JS validity checks using the JavaScript parser.
use swc_common::{FileName, SourceMap, Spanned};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};

pub(crate) enum ScriptError {
    ParserRejected {
        line: usize,
        column: usize,
        message: String,
    },
}

pub(crate) fn script(js: &str) -> core::result::Result<(), ScriptError> {
    let cm = SourceMap::default();
    let fm = cm.new_source_file(FileName::Custom("inline.js".into()).into(), js.to_string());
    let mut parser = Parser::new(
        Syntax::Es(EsSyntax::default()),
        StringInput::from(&*fm),
        None,
    );
    match parser.parse_script() {
        Ok(_) => Ok(()),
        Err(err) => {
            let loc = cm.lookup_char_pos(err.span().lo());
            Err(ScriptError::ParserRejected {
                line: loc.line,
                column: loc.col_display + 1,
                message: err.kind().msg().into_owned(),
            })
        }
    }
}
