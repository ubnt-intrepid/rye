fn case_sync() {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if rye::_internal::is_target(0u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        }
    }

    static TEST_CASE: rye::_internal::TestCase = rye::_internal::TestCase {
        sections: &[
            rye::_internal::Section::new(0u64, "resizing bigger changes size and capacity", true, rye::_internal::phf_set!())
        ],
    };
    TEST_CASE.run(__inner__);
}
