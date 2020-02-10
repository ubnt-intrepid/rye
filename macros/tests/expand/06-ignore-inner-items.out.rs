fn ignore_inner_items(suite: &mut rye::TestSuite<'_>) {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    static TEST_DESC: rye::_internal::TestDesc = rye::_internal::TestDesc {
        name: "ignore_inner_items",
        module_path: module_path!(),
        sections: &[
            rye::_internal::Section::new(0u64, "ignore_inner_items", true, rye::_internal::phf_set!())
        ],
    };
    suite.register(&TEST_DESC, __inner__);
}
