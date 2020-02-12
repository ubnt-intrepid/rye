fn ignore_inner_items(suite: &mut rye::_internal::TestSuite<'_>) {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    let desc = rye::_internal::TestDesc {
        name: "ignore_inner_items",
        module_path: module_path!(),
        ignored: false,
        sections: rye::_internal::hashmap! {},
        leaf_sections: &[],
    };
    suite.register(desc, __inner__);
}
