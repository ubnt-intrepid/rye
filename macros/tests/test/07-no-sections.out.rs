fn no_sections(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    __suite.add_test(::rye::_internal::Test {
        desc: ::rye::_internal::TestDesc {
            name: ::rye::_internal::test_name(::rye::_internal::module_path!(), "no_sections"),
            sections: ::rye::_internal::hashmap! {},
            leaf_sections: ::rye::_internal::vec![],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    ::rye::_internal::Result::Ok(())
}
