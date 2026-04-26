use maud::{DOCTYPE, html};
use maud_extensions::{css, js};

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
fn reset_surface_supports_inline_and_named_helpers() {
    css!(card_css, {
        me { gap: px!(12); }
    });

    js!(card_js, once, {
        me().class_add("ready");
    });

    let html = html! {
        article class="card" {
            (DOCTYPE)
            (css! {
                me { padding: rem!(1); }
            })
            (js! {
                me().class_add("inline-ready");
            })
            (card_css())
            (card_js())
        }
    }
    .into_string();

    assert!(html.contains("padding:1rem"));
    assert!(html.contains("inline-ready"));
    assert!(html.contains("gap:12px"));
    assert!(html.contains("data-mx-js-ran"));
    assert_marker_order(&html, &["padding:1rem", "inline-ready", "gap:12px"]);
}

#[test]
fn css_macros_allow_raw_css_escape_hatches() {
    css!(
        tokens_css,
        raw!(r#":root { --font-display: 'Newsreader', Georgia, serif; }"#)
    );

    let helper_html = tokens_css().into_string();
    assert!(helper_html.contains(":root"));
    assert!(helper_html.contains("'Newsreader', Georgia, serif"));

    let inline_html = html! {
        (css! {
            raw!(r#"[data-theme='light'] { --font-display: 'Newsreader', Georgia, serif; }"#)
        })
    }
    .into_string();

    assert!(inline_html.contains("[data-theme='light']"));
    assert!(inline_html.contains("'Newsreader', Georgia, serif"));
}

#[test]
fn css_macros_support_at_rules_and_unit_helpers() {
    let html = html! {
        (css! {
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
fn js_escapes_embedded_script_end_tags() {
    let html = html! {
        (js!(r#"console.log("</script>");"#))
    }
    .into_string();

    assert!(html.contains("console.log(\"<\\/script>\");"));
}

#[test]
fn js_escapes_embedded_script_end_tags_case_insensitively() {
    let html = html! {
        (js!(r#"console.log("</SCRIPT>");"#))
    }
    .into_string();

    assert!(html.contains("console.log(\"<\\/SCRIPT>\");"));
}

#[test]
fn css_escapes_embedded_style_end_tags_case_insensitively() {
    let html = html! {
        (css!(r#"body::before { content: "</STYLE>"; }"#))
    }
    .into_string();

    assert!(html.contains("content: \"<\\/STYLE>\";"));
}
