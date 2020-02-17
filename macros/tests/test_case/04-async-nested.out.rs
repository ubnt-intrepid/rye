fn case_async_nested(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::std::result::Result<(), ::rye::_internal::RegistryError> {
    async fn __inner__() {
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
    __suite.add_test_case(::rye::_internal::TestCase {
        desc: ::rye::_internal::TestDesc {
            name: "case_async_nested",
            module_path: ::rye::_internal::module_path!(),
            ignored: false,
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
            leaf_sections: &[ 1u64 ],
        },
        test_fn: ::rye::_internal::TestFn::AsyncTest(|| Box::pin(__inner__())),
    })?;
    Ok(())
}
