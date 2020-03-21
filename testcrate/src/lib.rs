rye::test_harness! {
    #![test_runner(crate::runner)]
    #![test_cases(test_case)]
}

#[cfg(test)]
fn runner(_: &[&dyn rye::test::TestSet]) {
    println!("hello")
}

#[rye::test]
fn test_case() {
    assert!(true);
}
