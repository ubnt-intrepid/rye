fn no_sections(__suite: &mut ::rye::_internal::TestSuite<'_>) {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    __suite.add_test_case(::rye::_internal::TestCase {
        desc: ::rye::_internal::TestDesc {
            name: "no_sections",
            module_path: ::rye::_internal::module_path!(),
            ignored: false,
            sections: ::rye::_internal::hashmap! {},
            leaf_sections: &[],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    });
}
