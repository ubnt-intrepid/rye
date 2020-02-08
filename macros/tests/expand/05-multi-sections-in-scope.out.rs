fn multi_section_in_scope() {
    fn __inner__() {
        if let Some(mut __section) = rye::_internal::new_section(0u64, "section1") {
            __section.scope(|| {
                assert!(1 + 1 == 2);
            });
        }
        if let Some(mut __section) = rye::_internal::new_section(1u64, "section2") {
            __section.scope(|| {
                assert!(1 + 1 == 2);
                if let Some(mut __section) = rye::_internal::new_section(2u64, "section2-1") {
                    __section.scope(|| {
                        assert!(true);
                        if let Some(mut __section) =
                            rye::_internal::new_section(3u64, "section2-1-2")
                        {
                            __section.scope(|| {
                                assert!(true);
                            });
                        }
                    });
                }
                if let Some(mut __section) = rye::_internal::new_section(4u64, "section2-2") {
                    __section.scope(|| {
                        assert!(true);
                    });
                }
                assert!(1 + 2 == 3);
            });
        }
        if let Some(mut __section) = rye::_internal::new_section(5u64, "section3") {
            __section.scope(|| {
                assert!(false);
            });
        }
    }
    rye::_internal::run(__inner__);
}
