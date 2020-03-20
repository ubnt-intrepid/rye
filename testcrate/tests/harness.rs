rye::test_harness! {
    #![test_runner(crate::runner)]
}

fn runner(_: &[&dyn rye::test::TestSet]) {
    println!("hello")
}

#[rye::test]
fn test_case() {
    assert!(true);
}
