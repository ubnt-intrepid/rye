async fn case_async() {
    async fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if rye::_internal::is_target(1u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        }
    }
    static TEST_CASE: rye::_internal::TestCase = rye::_internal::TestCase {
        sections: &[
            rye::_internal::Section::new(0u64, "case_async", false, rye::_internal::phf_set!())
          , rye::_internal::Section::new(1u64, "resizing bigger changes size and capacity", true, rye::_internal::phf_set!(0u64))
        ],
    };
    TEST_CASE.run_async(__inner__).await;
}
