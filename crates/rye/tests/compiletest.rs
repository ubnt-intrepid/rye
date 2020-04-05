#[rustversion::stable]
#[test]
fn compiletest() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile-fail/*.rs");
}
