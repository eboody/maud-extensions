use maud::html;
use maud_extensions::{signals_inline, surreal_scope_inline, surreal_scope_signals_inline};

#[test]
fn surreal_scope_inline_emits_surreal_and_css_scope_runtime() {
    let markup = html! { head { (surreal_scope_inline!()) } }.into_string();

    assert!(markup.contains("Surreal 1.3.4"));
    assert!(markup.contains("css-scope-inline"));
    assert!(!markup.contains("preactSignalsCore"));
}

#[test]
fn signals_inline_emits_signals_runtime_and_adapter() {
    let markup = html! { head { (signals_inline!()) } }.into_string();

    assert!(markup.contains("preactSignalsCore"));
    assert!(markup.contains("Signals Adapter"));
    assert!(!markup.contains("Surreal 1.3.4"));
}

#[test]
fn surreal_scope_signals_inline_emits_all_runtimes_in_order() {
    let markup = html! { head { (surreal_scope_signals_inline!()) } }.into_string();

    let surreal_index = markup.find("Surreal 1.3.4").unwrap();
    let scope_index = markup.find("css-scope-inline").unwrap();
    let signals_index = markup.find("preactSignalsCore").unwrap();
    let adapter_index = markup.find("Signals Adapter").unwrap();

    assert!(surreal_index < scope_index);
    assert!(scope_index < signals_index);
    assert!(signals_index < adapter_index);
}
