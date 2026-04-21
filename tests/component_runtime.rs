use maud::{DOCTYPE, Markup, html};
use maud_extensions::{
    component, css, css_file, inline_css, inline_js, js, js_file, signals_inline,
    surreal_scope_inline, surreal_scope_signals_inline,
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

fn attribute_value<'a>(html: &'a str, attr: &str) -> &'a str {
    let marker = format!("{attr}=\"");
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("missing attribute: {attr}"))
        + marker.len();
    let end = html[start..]
        .find('"')
        .unwrap_or_else(|| panic!("unterminated attribute: {attr}"));
    &html[start..start + end]
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
        "<article class=\"status-card\" data-mx-component=\"\" data-mx-css-scope=\"mx-css-"
    ));
    assert!(html.contains("<p class=\"message\">ok</p>"));
    assert!(html.contains("<script>"));
    assert!(html.contains("data-mx-js-mode"));
    assert!(html.contains("<style data-mx-css-id="));
    assert_marker_order(&html, &["<p class=\"message\">ok</p>", "<style data-mx-css-id=", "<script>"]);
    assert_eq!(
        attribute_value(&html, "data-mx-css-scope"),
        attribute_value(&html, "data-mx-css-id")
    );
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
    assert!(html.contains("<div data-mx-component=\"\" data-mx-css-scope=\"mx-css-"));
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
fn css_macros_allow_raw_css_escape_hatches() {
    fn token_view() -> Markup {
        css! {
            me { color: #111; }
            raw!(r#"[data-theme='light'] { --font-display: 'Newsreader', Georgia, serif; }"#)
        }

        css()
    }

    let helper_html = token_view().into_string();
    assert!(helper_html.contains("[data-theme='light']"));
    assert!(helper_html.contains("'Newsreader', Georgia, serif"));

    let inline_html = html! {
        (inline_css! {
            raw!(r#":root { --font-display: 'Newsreader', Georgia, serif; }"#)
        })
    }
    .into_string();

    assert!(inline_html.contains(":root"));
    assert!(inline_html.contains("'Newsreader', Georgia, serif"));
}

#[test]
fn css_macros_support_at_rules_and_unit_helpers() {
    let html = html! {
        (inline_css! {
            media!("(min-width: 48rem)", {
                me { padding: rem!(2); }
            })
            container!("card (min-width: 30rem)", {
                me { gap: px!(12); }
            })
            supports!("(display: grid)", {
                me { width: pct!(100); }
            })
            layer!(components, {
                me { margin: em!(1.5); }
            })
            keyframes!(fade_in, {
                from { opacity: 0; }
                to { opacity: 1; }
            })
        })
    }
    .into_string();

    assert!(html.contains("@media (min-width: 48rem)"));
    assert!(html.contains("padding:2rem"));
    assert!(html.contains("@container card (min-width: 30rem)"));
    assert!(html.contains("gap:12px"));
    assert!(html.contains("@supports (display: grid)"));
    assert!(html.contains("width:100%"));
    assert!(html.contains("@layer components"));
    assert!(html.contains("margin:1.5em"));
    assert!(html.contains("@keyframes fade_in"));
    assert!(html.contains("opacity:1"));
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
    assert!(html.contains("<script>me().style.color = 'red'<\\/script>"));
    assert!(html.contains("function scopePendingStyles()"));
    assert!(html.contains("scopePendingStyles()"));
    assert!(html.contains("document?.querySelectorAll('style[data-mx-css-id]:not([ready])')"));
    assert!(html.contains("[data-mx-css-scope=\"") );
}

#[test]
fn inline_js_escapes_embedded_script_end_tags() {
    let html = html! {
        (inline_js!(r#"console.log("</script>");"#))
    }
    .into_string();

    assert!(html.contains("console.log(\"<\\/script>\");"));
}

#[test]
fn inline_js_escapes_embedded_script_end_tags_case_insensitively() {
    let html = html! {
        (inline_js!(r#"console.log("</SCRIPT>");"#))
    }
    .into_string();

    assert!(html.contains("console.log(\"<\\/SCRIPT>\");"));
}

#[test]
fn inline_css_escapes_embedded_style_end_tags_case_insensitively() {
    let html = html! {
        (inline_css!(r#"body::before { content: "</STYLE>"; }"#))
    }
    .into_string();

    assert!(html.contains("content: \"<\\/STYLE>\";"));
}

#[test]
fn file_macros_escape_embedded_raw_text_end_tags_case_insensitively() {
    let html = html! {
        (js_file!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/raw_text_end_tag.js"
        )))
        (css_file!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/raw_text_end_tag.css"
        )))
    }
    .into_string();

    assert!(html.contains("console.log(\"<\\/SCRIPT>\");"));
    assert!(html.contains("content: \"<\\/STYLE>\";"));
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
        "<div class=\"empty-helpers\" data-mx-component=\"\" data-mx-css-scope=\"mx-css-"
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
        once_html.contains("class=\"once-mode\" data-mx-component=\"\" data-mx-css-scope=\"mx-css-")
    );
    assert!(
        always_html
            .contains("class=\"always-mode\" data-mx-component=\"\" data-mx-css-scope=\"mx-css-")
    );
    assert_marker_order(
        &once_html,
        &[
            "me().class_add(\"ready\");",
            "setAttribute",
        ],
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
