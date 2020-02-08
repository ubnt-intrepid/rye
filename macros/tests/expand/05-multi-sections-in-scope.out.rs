fn multi_section_in_scope() {
    fn __inner__(__section: &rye::_internal::Section) {
        if __section.is_target(0u64) {
            assert!(1 + 1 == 2);
        }
        if __section.is_target(1u64) {
            assert!(1 + 1 == 2);
            if __section.is_target(2u64) {
                assert!(true);
                if __section.is_target(3u64) {
                    assert!(true);
                }
            }
            if __section.is_target(4u64) {
                assert!(true);
            }
            assert!(1 + 2 == 3);
        }
        if __section.is_target(5u64) {
            assert!(false);
        }
    }
    static SECTIONS: &[rye::_internal::Section] = &[
        rye::_internal::Section::new(0u64, "section1", true, rye::_internal::phf_set!())
      , rye::_internal::Section::new(1u64, "section2", false, rye::_internal::phf_set!())
      , rye::_internal::Section::new(2u64, "section2-1", false, rye::_internal::phf_set!(1u64))
      , rye::_internal::Section::new(3u64, "section2-1-2", true, rye::_internal::phf_set!(1u64, 2u64))
      , rye::_internal::Section::new(4u64, "section2-2", true, rye::_internal::phf_set!(1u64))
      , rye::_internal::Section::new(5u64, "section3", true, rye::_internal::phf_set!())
    ];
    rye::_internal::run(__inner__, SECTIONS);
}
