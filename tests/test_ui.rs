#[test]
#[ignore] // disabled by default to avoid failures in cargo-tarpaulin tests.
fn ui_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-passes.rs");
}
