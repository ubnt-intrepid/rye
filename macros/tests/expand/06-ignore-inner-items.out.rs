fn ignore_inner_items() {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    
    static TEST_CASE: rye::_internal::TestCase = rye::_internal::TestCase {
        sections: &[],
    };
    TEST_CASE.run(__inner__);
}
