fn multi_section_in_scope(suite: &mut rye::TestSuite<'_>) {
    fn __inner__() {
        if rye::_internal::is_target(1u64) {
            assert!(1 + 1 == 2);
        }
        if rye::_internal::is_target(2u64) {
            assert!(1 + 1 == 2);
            if rye::_internal::is_target(3u64) {
                assert!(true);
                if rye::_internal::is_target(4u64) {
                    assert!(true);
                }
            }
            if rye::_internal::is_target(5u64) {
                assert!(true);
            }
            assert!(1 + 2 == 3);
        }
        if rye::_internal::is_target(6u64) {
            assert!(false);
        }
    }
    static TEST_DESC: rye::_internal::TestDesc = rye::_internal::TestDesc {
        name: "multi_section_in_scope",
        module_path: module_path!(),
        sections: &[
            rye::_internal::Section::new(0u64, "multi_section_in_scope", false, rye::_internal::phf_set!())
          , rye::_internal::Section::new(1u64, "section1", true, rye::_internal::phf_set!(0u64))
          , rye::_internal::Section::new(2u64, "section2", false, rye::_internal::phf_set!(0u64))
          , rye::_internal::Section::new(3u64, "section2-1", false, rye::_internal::phf_set!(0u64, 2u64))
          , rye::_internal::Section::new(4u64, "section2-1-2", true, rye::_internal::phf_set!(0u64, 2u64, 3u64))
          , rye::_internal::Section::new(5u64, "section2-2", true, rye::_internal::phf_set!(0u64, 2u64))
          , rye::_internal::Section::new(6u64, "section3", true, rye::_internal::phf_set!(0u64))
        ],
    };
    suite.register(&TEST_DESC, __inner__);
}
