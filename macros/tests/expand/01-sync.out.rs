fn case_sync(suite: &mut rye::TestSuite<'_>) {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if rye::_internal::is_target(1u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        }
    }
    static TEST_DESC: rye::_internal::TestDesc = rye::_internal::TestDesc {
        name: "case_sync",
        module_path: module_path!(),
        sections: &[
            rye::_internal::Section::new(0u64, "case_sync", false, rye::_internal::phf_set!())
          , rye::_internal::Section::new(1u64, "resizing bigger changes size and capacity", true, rye::_internal::phf_set!(0u64))
        ],
    };
    suite.register(&TEST_DESC, __inner__);
}
