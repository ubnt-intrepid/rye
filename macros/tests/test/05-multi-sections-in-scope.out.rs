fn multi_section_in_scope() {
    ::rye::__enter_section!(0u64, {
        assert!(1 + 1 == 2);
    });

    ::rye::__enter_section!(1u64, {
        assert!(1 + 1 == 2);

        ::rye::__enter_section!(2u64, {
            assert!(true);

            ::rye::__enter_section!(3u64, {
                assert!(true);
            });
        });

        ::rye::__enter_section!(4u64, {
            assert!(true);
        });

        assert!(1 + 2 == 3);
    });

    ::rye::__enter_section!(5u64, {
        assert!(false);
    });
}

::rye::__declare_test_module! {
    name = multi_section_in_scope;
    sections = {
        0u64 => ("section1"     , {});
        1u64 => ("section2"     , {});
        2u64 => ("section2-1"   , { 1u64 });
        3u64 => ("section2-1-2" , { 1u64, 2u64 });
        4u64 => ("section2-2"   , { 1u64 });
        5u64 => ("section3"     , {});
    };
    leaf_sections = { 0u64, 3u64, 4u64, 5u64 };
    [blocking] test_fn = multi_section_in_scope;
}
