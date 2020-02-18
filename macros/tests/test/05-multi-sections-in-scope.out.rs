fn multi_section_in_scope(__suite: &mut ::rye::_internal::Registry<'_>)
    -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
    fn __inner__() {
        if ::rye::_internal::is_target(0u64) {
            assert!(1 + 1 == 2);
        }
        if ::rye::_internal::is_target(1u64) {
            assert!(1 + 1 == 2);
            if ::rye::_internal::is_target(2u64) {
                assert!(true);
                if ::rye::_internal::is_target(3u64) {
                    assert!(true);
                }
            }
            if ::rye::_internal::is_target(4u64) {
                assert!(true);
            }
            assert!(1 + 2 == 3);
        }
        if ::rye::_internal::is_target(5u64) {
            assert!(false);
        }
    }
    __suite.add_test(::rye::_internal::Test {
        desc: ::rye::_internal::TestDesc {
            name: ::rye::_internal::test_name(::rye::_internal::module_path!(), "multi_section_in_scope"),
            sections: ::rye::_internal::hashmap! {
                0u64 => ::rye::_internal::Section { name: "section1"     , ancestors: ::rye::_internal::hashset!()           , },
                1u64 => ::rye::_internal::Section { name: "section2"     , ancestors: ::rye::_internal::hashset!()           , },
                2u64 => ::rye::_internal::Section { name: "section2-1"   , ancestors: ::rye::_internal::hashset!(1u64)       , },
                3u64 => ::rye::_internal::Section { name: "section2-1-2" , ancestors: ::rye::_internal::hashset!(1u64, 2u64) , },
                4u64 => ::rye::_internal::Section { name: "section2-2"   , ancestors: ::rye::_internal::hashset!(1u64)       , },
                5u64 => ::rye::_internal::Section { name: "section3"     , ancestors: ::rye::_internal::hashset!()           , },
            },
            leaf_sections: ::rye::_internal::vec![ 0u64, 3u64, 4u64, 5u64 ],
        },
        test_fn: ::rye::_internal::TestFn::SyncTest(__inner__),
    })?;
    ::rye::_internal::Result::Ok(())
}
