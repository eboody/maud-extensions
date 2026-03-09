#[test]
fn component_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/component_valid.rs");
    t.compile_fail("tests/ui/component_empty.rs");
    t.compile_fail("tests/ui/component_multiple_roots.rs");
    t.compile_fail("tests/ui/component_control_flow_root.rs");
    t.compile_fail("tests/ui/component_missing_helpers.rs");
    t.compile_fail("tests/ui/component_missing_js_helper.rs");
    t.compile_fail("tests/ui/component_missing_css_helper.rs");
}
