use maud::html;
use maud_extensions::Init;

#[test]
fn init_all_emits_all_runtimes_in_order() {
    let markup = html! { head { (Init::all()) } }.into_string();

    let surreal_index = markup.find("Surreal 1.3.4").unwrap();
    let scope_index = markup.find("css-scope-inline").unwrap();
    let signals_index = markup.find("preactSignalsCore").unwrap();
    let adapter_index = markup.find("Signals Adapter").unwrap();

    assert!(surreal_index < scope_index);
    assert!(scope_index < signals_index);
    assert!(signals_index < adapter_index);
}

#[test]
fn init_builder_emits_selected_runtimes_only_once() {
    let markup = html! {
        head {
            (Init::new().signals().signals().surrealjs().scoped_css().build())
        }
    }
    .into_string();

    assert_eq!(markup.matches("Surreal 1.3.4").count(), 1);
    assert_eq!(markup.matches("css-scope-inline").count(), 1);
    assert_eq!(markup.matches("Signals Adapter").count(), 1);
    let surreal_index = markup.find("Surreal 1.3.4").unwrap();
    let scope_index = markup.find("css-scope-inline").unwrap();
    let signals_index = markup.find("preactSignalsCore").unwrap();
    let adapter_index = markup.find("Signals Adapter").unwrap();

    assert!(surreal_index < scope_index);
    assert!(scope_index < signals_index);
    assert!(signals_index < adapter_index);
}
