// JS validity checks using the JavaScript parser.
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax};

pub(crate) fn script(js: &str) -> core::result::Result<(), String> {
    let cm = SourceMap::default();
    let fm = cm.new_source_file(FileName::Custom("inline.js".into()).into(), js.to_string());
    let mut parser = Parser::new(
        Syntax::Es(EsSyntax::default()),
        StringInput::from(&*fm),
        None,
    );
    match parser.parse_script() {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("js! could not parse JavaScript: {err:?}")),
    }
}
