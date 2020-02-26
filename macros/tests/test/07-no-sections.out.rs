fn no_sections() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}

::rye::__declare_test_module! {
    name = no_sections;
    sections = {};
    leaf_sections = {};
    [blocking] test_fn = no_sections;
}
