fn ignore_inner_items(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::std::result::Result<(), ::rye::_internal::RegistryError> {
    fn __inner__() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }
    __suite.add_test_case(::rye::_internal::TestCase {
        desc: ::rye::_internal::TestDesc {
            name: "ignore_inner_items",
            module_path: ::rye::_internal::module_path!(),
            ignored: false,
            sections: ::rye::_internal::hashmap! {},
            leaf_sections: &[],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    Ok(())
}
