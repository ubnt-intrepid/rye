fn case_sync_nested(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if ::rye::_internal::is_target(0u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 10);

            if ::rye::_internal::is_target(1u64) {
                vec.resize(0, 0);
                assert_eq!(vec.len(), 0);
                assert!(vec.capacity() >= 10);
            }
        }
    }
    __suite.add_test(::rye::_internal::Test {
        desc: ::rye::_internal::TestDesc {
            name: ::rye::_internal::test_name(::rye::_internal::module_path!(), "case_sync_nested"),
            sections: ::rye::_internal::hashmap! {
                0u64 => ::rye::_internal::Section {
                    name: "resizing bigger changes size and capacity",
                    ancestors: ::rye::_internal::hashset!(),
                },
                1u64 => ::rye::_internal::Section {
                    name: "shrinking smaller does not changes capacity",
                    ancestors: ::rye::_internal::hashset!(0u64),
                },
            },
            leaf_sections: ::rye::_internal::vec![ 1u64 ],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    ::rye::_internal::Result::Ok(())
}
