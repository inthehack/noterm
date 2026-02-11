#[test]
fn it_should_expand_print_macro_with_valid_input() {
    let cases = trybuild::TestCases::new();
    cases.pass("src/macros/ui/001-expand-valid-input-no-arg.rs");
    cases.pass("src/macros/ui/002-expand-valid-input-with-args.rs");
    cases.compile_fail("src/macros/ui/003-fail-expand-invalid-input-missing-fmt.rs");
    cases.compile_fail("src/macros/ui/004-fail-expand-invalid-input-missing-arg.rs");
}
