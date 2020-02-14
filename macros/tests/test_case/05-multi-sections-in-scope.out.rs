fn multi_section_in_scope(suite: &mut ::rye::_internal::TestSuite<'_>) {
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
    let desc = ::rye::_internal::TestDesc {
        name: "multi_section_in_scope",
        module_path: module_path!(),
        ignored: false,
        sections: ::rye::_internal::hashmap! {
            0u64 => ::rye::_internal::Section::new("section1", ::rye::_internal::hashset!()),
            1u64 => ::rye::_internal::Section::new("section2", ::rye::_internal::hashset!()),
            2u64 => ::rye::_internal::Section::new("section2-1", ::rye::_internal::hashset!(1u64)),
            3u64 => ::rye::_internal::Section::new("section2-1-2", ::rye::_internal::hashset!(1u64, 2u64)),
            4u64 => ::rye::_internal::Section::new("section2-2", ::rye::_internal::hashset!(1u64)),
            5u64 => ::rye::_internal::Section::new("section3", ::rye::_internal::hashset!()),
        },
        leaf_sections: &[ 0u64, 3u64, 4u64, 5u64 ],
    };
    suite.register(desc, __inner__);
}
