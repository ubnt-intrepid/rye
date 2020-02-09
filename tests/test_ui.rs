#[test]
#[ignore] // disabled by default to avoid failures in cargo-tarpaulin tests.
fn ui_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-passes.rs");
    t.compile_fail("tests/ui/02-wrong-section.rs");
    t.compile_fail("tests/ui/03-section-in-loop.rs");
    t.compile_fail("tests/ui/04-section-in-closure.rs");
    t.compile_fail("tests/ui/05-section-in-async-block.rs");
}
