fn ignore_inner_items(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    __suite.add_test(::rye::_internal::Test {
        desc: ::rye::_internal::TestDesc {
            name: ::rye::_internal::test_name(::rye::_internal::module_path!(), "ignore_inner_items"),
            sections: ::rye::_internal::hashmap! {},
            leaf_sections: ::rye::_internal::vec![],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    ::rye::_internal::Result::Ok(())
}
