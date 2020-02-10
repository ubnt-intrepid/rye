fn ignore_inner_items() {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    static TEST_CASE: rye::_internal::TestCase = rye::_internal::TestCase {
        sections: &[
            rye::_internal::Section::new(0u64, "ignore_inner_items", true, rye::_internal::phf_set!())
        ],
    };
    TEST_CASE.run(__inner__);
}
