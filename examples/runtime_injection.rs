use maud_extensions::{
    css_file, js_file, signals_inline, surreal_scope_inline, surreal_scope_signals_inline,
};

fn main() {
    let _bundled = maud::html! {
        head { (surreal_scope_inline!()) }
    };

    let _signals_only = maud::html! {
        head { (signals_inline!()) }
    };

    let _bundled_with_signals = maud::html! {
        head { (surreal_scope_signals_inline!()) }
    };

    let _manual_composed = maud::html! {
        head {
            // When composing manually, keep the Surreal runtime first so the
            // Signals adapter can patch `me(...)` before component scripts run.
            (surreal_scope_inline!())
            (signals_inline!())
        }
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
