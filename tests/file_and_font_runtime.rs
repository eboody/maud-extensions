use maud::html;
use maud_extensions::{css_file, font_face, font_faces, js_file};

#[test]
fn file_macros_inline_fixture_assets() {
    let html = html! {
        (js_file!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/runtime.js"
        )))
        (css_file!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/runtime.css"
        )))
    }
    .into_string();

    assert!(html.contains("fixture-ready"));
    assert!(html.contains(".fixture"));
    assert!(html.contains("display: block"));
}

#[test]
fn font_face_embeds_a_single_face_without_extra_dependencies() {
    let html = html! {
        style {
            (font_face!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/demo-font.woff2"
                ),
                "Fixture Sans"
            ))
        }
    }
    .into_string();

    assert!(html.contains("@font-face"));
    assert!(html.contains("font-family: \"Fixture Sans\""));
    assert!(html.contains("data:font/woff2;base64,"));
}

#[test]
fn font_face_escapes_family_names_for_css_strings() {
    let html = html! {
        style {
            (font_face!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/demo-font.woff2"
                ),
                "O'Reilly \"Sans\""
            ))
        }
    }
    .into_string();

    assert!(html.contains("font-family: \"O'Reilly \\\"Sans\\\"\""));
}

#[test]
fn font_faces_concatenates_multiple_faces() {
    let html = html! {
        style {
            (font_faces!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/demo-font.woff2"
                ), "Fixture Sans";
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/demo-font-bold.woff2"
                ), "Fixture Sans", "700", "italic"
            ))
        }
    }
    .into_string();

    assert_eq!(html.matches("@font-face").count(), 2);
    assert!(html.contains("font-style: italic"));
    assert!(html.contains("font-weight: 700"));
}

#[test]
fn font_face_detects_extensions_case_insensitively() {
    let html = html! {
        style {
            (font_face!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/demo-font-upper.WOFF2"
                ),
                "Uppercase Sans"
            ))
        }
    }
    .into_string();

    assert!(html.contains("data:font/woff2;base64,"));
}
