#[allow(missing_docs)]
fn attributes() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}

path::to::rye::__declare_test_module! {
    name = attributes;
    sections = {};
    leaf_sections = {};
    [blocking] test_fn = attributes;
}
