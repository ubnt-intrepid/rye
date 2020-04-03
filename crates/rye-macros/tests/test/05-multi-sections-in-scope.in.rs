fn multi_section_in_scope(ctx: &mut Context<'_>) {
    section!(ctx, "section1", {
        assert!(1 + 1 == 2);
    });

    section!(ctx, "section2", {
        assert!(1 + 1 == 2);

        section!(ctx, "section2-1", {
            assert!(true);

            section!(ctx, "section2-1-2", {
                assert!(true);
            });
        });

        section!(ctx, "section2-2", {
            assert!(true);
        });

        assert!(1 + 2 == 3);
    });

    section!(ctx, "section3", {
        assert!(false);
    });
}
