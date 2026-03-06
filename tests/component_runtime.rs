use maud::{DOCTYPE, Markup, html};
use maud_extensions::{component, css, inline_css, inline_js, js, surreal_scope_inline};

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
    assert!(html.contains("<article class=\"status-card\">"));
    assert!(html.contains("<p class=\"message\">ok</p>"));
    assert!(html.contains("<script>"));
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
    assert!(html.contains("<div>trailing"));
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
}
