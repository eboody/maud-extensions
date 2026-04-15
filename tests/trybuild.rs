#[test]
fn component_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/component_valid.rs");
    t.pass("tests/ui/component_directive_valid.rs");
    t.pass("tests/ui/component_dynamic_attr_valid.rs");
    t.pass("tests/ui/slot_valid.rs");
    t.compile_fail("tests/ui/component_empty.rs");
    t.compile_fail("tests/ui/component_multiple_roots.rs");
    t.compile_fail("tests/ui/component_control_flow_root.rs");
    t.compile_fail("tests/ui/component_missing_helpers.rs");
    t.compile_fail("tests/ui/component_missing_js_helper.rs");
    t.compile_fail("tests/ui/component_missing_css_helper.rs");
    t.compile_fail("tests/ui/component_unknown_directive.rs");
    t.compile_fail("tests/ui/component_conflicting_directives.rs");
    t.compile_fail("tests/ui/component_directive_after_root.rs");
    t.compile_fail("tests/ui/component_trailing_tokens.rs");
}
