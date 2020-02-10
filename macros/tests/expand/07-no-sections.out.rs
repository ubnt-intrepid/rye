fn no_sections(suite: &mut rye::TestSuite<'_>) {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    static TEST_DESC: rye::_internal::TestDesc = rye::_internal::TestDesc {
        name: "no_sections",
        module_path: module_path!(),
        sections: &[
            rye::_internal::Section::new(0u64, "no_sections", true, rye::_internal::phf_set!())
        ],
    };
    suite.register(&TEST_DESC, __inner__);
}
