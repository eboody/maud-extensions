#[test]
fn reset_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/css_macro_helpers_valid.rs");
    t.pass("tests/ui/css_named_helper_valid.rs");
    t.pass("tests/ui/css_raw_valid.rs");
    t.compile_fail("tests/ui/css_raw_invalid_argument.rs");
    t.compile_fail("tests/ui/css_macro_helpers_invalid.rs");
    t.compile_fail("tests/ui/css_invalid_stylesheet.rs");
    t.compile_fail("tests/ui/css_invalid_helper_name.rs");
    t.compile_fail("tests/ui/css_named_helper_trailing_tokens.rs");
    t.compile_fail("tests/ui/js_invalid_helper_name.rs");
    t.compile_fail("tests/ui/js_invalid_mode.rs");
    t.compile_fail("tests/ui/js_invalid_script.rs");
    t.compile_fail("tests/ui/js_named_helper_trailing_tokens.rs");
}
