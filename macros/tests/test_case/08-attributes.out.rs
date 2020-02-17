fn ignored(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::std::result::Result<(), ::rye::_internal::RegistryError> {
    #[allow(unused_variables)]
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    __suite.add_test_case(::rye::_internal::TestCase {
        desc: ::rye::_internal::TestDesc {
            name: "ignored",
            module_path: ::rye::_internal::module_path!(),
            ignored: true,
            sections: ::rye::_internal::hashmap! {},
            leaf_sections: &[],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    Ok(())
}
