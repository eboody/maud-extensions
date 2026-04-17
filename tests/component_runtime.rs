use maud::{DOCTYPE, Markup, html};
use maud_extensions::{
    component, css, inline_css, inline_js, js, signals_inline, surreal_scope_inline,
    surreal_scope_signals_inline,
};

fn assert_marker_order(haystack: &str, markers: &[&str]) {
    let mut last = 0usize;
    for marker in markers {
        let index = haystack
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker: {marker}"));
        assert!(index >= last, "marker `{marker}` appeared out of order");
        last = index;
    }
}

#[test]
fn component_injects_js_and_css_helpers_inside_root() {
    fn status_card(message: &str) -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            article class="status-card" {
                h2 { "System status" }
                p class="message" { (message) }
            }
        };

        css! {
            me { border: 1px solid #ddd; }
            me.ready { border-color: #16a34a; }
        }

        view
    }

    let html = status_card("ok").into_string();
    assert!(html.contains(
        "<article class=\"status-card\" data-mx-component=\"\" data-mx-js-mode=\"always\">"
    ));
    assert!(html.contains("<p class=\"message\">ok</p>"));
    assert!(html.contains("<script>"));
    assert!(html.contains("data-mx-js-mode"));
    assert!(html.contains("<style data-mx-css-id="));
}

#[test]
fn component_allows_trailing_semicolon() {
    fn trailing() -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            div { "trailing" };
        };

        css! {
            me { color: #111; }
        }

        view
    }

    let html = trailing().into_string();
    assert!(html.contains("<div data-mx-component=\"\" data-mx-js-mode=\"always\">trailing"));
    assert!(html.contains("<script>"));
    assert!(html.contains("<style data-mx-css-id="));
}

#[test]
fn inline_macros_emit_direct_tags() {
    let html = html! {
        (DOCTYPE)
        div {
            (inline_js! { me().class_add("pinged"); })
            (inline_css! { me { display: block; } })
        }
    }
    .into_string();

    assert!(html.contains("<script>"));
    assert!(html.contains("class_add"));
    assert!(html.contains("<style data-mx-css-id="));
    assert!(html.contains("display:block"));
}

#[test]
fn surreal_scope_inline_emits_bundled_scripts() {
    let html = html! {
        (surreal_scope_inline!())
    }
    .into_string();

    assert!(html.contains("<script>"));
    assert!(html.contains("Welcome to Surreal"));
    assert!(html.contains("CSS Scope Inline"));
    assert!(html.contains("mxCleanupByRoot"));
    assert!(html.contains("onWindow"));
    assert!(html.contains("observeMutations"));
}

#[test]
fn signals_inline_emits_signals_runtime_and_adapter() {
    let html = html! {
        (signals_inline!())
    }
    .into_string();

    assert!(html.contains("<script>"));
    assert!(html.contains("preactSignalsCore"));
    assert!(html.contains("Maud Extensions Signals Adapter"));
    assert!(html.contains("window.mx"));
    assert!(html.contains("bindText"));
    assert!(html.contains("bindShow"));
    assert!(html.contains("window.me = wrappedMe"));
    assert!(html.contains("window.any = wrappedAny"));
    assert_marker_order(
        &html,
        &["preactSignalsCore", "Maud Extensions Signals Adapter"],
    );
}

#[test]
fn surreal_scope_signals_inline_emits_all_bundled_scripts_in_order() {
    let html = html! {
        (surreal_scope_signals_inline!())
    }
    .into_string();

    assert!(html.contains("Welcome to Surreal"));
    assert!(html.contains("CSS Scope Inline"));
    assert!(html.contains("preactSignalsCore"));
    assert!(html.contains("Maud Extensions Signals Adapter"));
    assert!(html.contains("bindClass"));
    assert_marker_order(
        &html,
        &[
            "Welcome to Surreal",
            "CSS Scope Inline",
            "preactSignalsCore",
            "Maud Extensions Signals Adapter",
        ],
    );
}

#[test]
fn component_allows_empty_js_and_css_helpers() {
    fn empty_helpers() -> Markup {
        js! {}

        let view = component! {
            div class="empty-helpers" {
                "ok"
            }
        };

        css! {}

        view
    }

    let html = empty_helpers().into_string();
    assert!(html.contains(
        "<div class=\"empty-helpers\" data-mx-component=\"\" data-mx-js-mode=\"always\">"
    ));
    assert!(html.contains("<script>"));
    assert!(html.contains("data-mx-js-ran"));
    assert!(html.contains("<style data-mx-css-id="));
}

#[test]
fn component_supports_js_mode_directives() {
    fn once_mode() -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            @js-once
            section class="once-mode" {
                "once"
            }
        };

        css! {}

        view
    }

    fn always_mode() -> Markup {
        js! {
            me().class_add("ready");
        }

        let view = component! {
            @js-always
            section class="always-mode" {
                "always"
            }
        };

        css! {}

        view
    }

    let once_html = once_mode().into_string();
    let always_html = always_mode().into_string();

    assert!(
        once_html.contains("class=\"once-mode\" data-mx-component=\"\" data-mx-js-mode=\"once\"")
    );
    assert!(
        always_html
            .contains("class=\"always-mode\" data-mx-component=\"\" data-mx-js-mode=\"always\"")
    );
}

#[test]
fn js_literal_form_still_inlines_verbatim_js() {
    fn literal_js() -> Markup {
        js!("me().class_add('literal-ready');");

        let view = component! {
            div class="literal-js" {
                "ok"
            }
        };

        css! {}

        view
    }

    let html = literal_js().into_string();
    assert!(html.contains("literal-ready"));
}

#[test]
fn component_page_can_include_combined_signals_runtime() {
    fn counter() -> Markup {
        js! {
            const count = mx.signal(0);
            me(".count").bindText(count);
            me(".inc").on("click", () => count.value++);
        }

        let view = component! {
            @js-once
            section class="counter" {
                span class="count" {}
                button class="inc" type="button" { "+" }
            }
        };

        css! {}

        view
    }

    let html = html! {
        head {
            (surreal_scope_signals_inline!())
        }
        body {
            (counter())
        }
    }
    .into_string();

    assert!(html.contains("data-mx-component"));
    assert!(html.contains("mx.signal(0)"));
    assert!(html.contains(".bindText(count)"));
    assert!(html.contains("Maud Extensions Signals Adapter"));
    assert!(html.contains("preactSignalsCore"));
    assert_marker_order(
        &html,
        &[
            "Welcome to Surreal",
            "CSS Scope Inline",
            "preactSignalsCore",
            "Maud Extensions Signals Adapter",
        ],
    );
}

#[test]
fn named_css_helper_emits_style_markup() {
    fn named_css() -> Markup {
        css! { "card_border", {
            .card { border: 1px solid #ddd; }
        } }

        card_border()
    }

    let html = named_css().into_string();
    assert!(html.contains("<style data-mx-css-id="));
    assert!(html.contains(".card"));
    assert!(html.contains("border"));
    assert!(html.contains("#ddd"));
}
