fn multi_section_in_scope() {
    section!("section1", {
        assert!(1 + 1 == 2);
    });

    section!("section2", {
        assert!(1 + 1 == 2);

        section!("section2-1", {
            assert!(true);

            section!("section2-1-2", {
                assert!(true);
            });
        });

        section!("section2-2", {
            assert!(true);
        });

        assert!(1 + 2 == 3);
    });

    section!("section3", {
        assert!(false);
    });
}
