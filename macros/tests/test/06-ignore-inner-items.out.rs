fn ignore_inner_items() {
    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}

::rye::__declare_test_module! {
    name = ignore_inner_items;
    sections = {};
    leaf_sections = {};
    [blocking] test_fn = ignore_inner_items;
}
