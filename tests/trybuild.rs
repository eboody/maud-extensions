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

#[cfg(feature = "components")]
#[test]
fn component_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/component_tuple_struct.rs");
    t.compile_fail("tests/ui/component_default_slot_wrong_type.rs");
    t.compile_fail("tests/ui/component_named_slot_wrong_type.rs");
    t.compile_fail("tests/ui/component_each_non_vec.rs");
    t.compile_fail("tests/ui/component_repeated_slot_wrong_type.rs");
    t.compile_fail("tests/ui/component_multiple_slots_need_default.rs");
    t.compile_fail("tests/ui/component_legacy_slot_attr.rs");
    t.compile_fail("tests/ui/component_impl_multiple_roots.rs");
    t.compile_fail("tests/ui/component_impl_duplicate_css.rs");
    t.compile_fail("tests/ui/component_impl_duplicate_js.rs");
    t.compile_fail("tests/ui/component_impl_invalid_js_mode.rs");
}
