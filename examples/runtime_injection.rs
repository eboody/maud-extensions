use maud_extensions::{css_file, js_file, surreal_scope_inline};

fn main() {
    let _bundled = maud::html! {
        head { (surreal_scope_inline!()) }
    };

    let _custom = maud::html! {
        head {
            (js_file!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/runtime.js"
            )))
            (css_file!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/runtime.css"
            )))
        }
    };
}
