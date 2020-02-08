fn case_sync() {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        {
            static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                name: "resizing bigger changes size and capacity",
                file: file!(),
                line: line!(),
                column: column!(),
            };
            if let Some(mut __section) = rye::_internal::new_section(&SECTION) {
                rye::_internal::with_section(&mut __section, || {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 5);
                });
            }
        }
    }

    #[allow(unused_mut)]
    let mut test_case = rye::_internal::TestCase::new();
    while !test_case.completed() {
        let mut section = test_case.root_section();
        rye::_internal::with_section(&mut section, __inner__);
    }
}
