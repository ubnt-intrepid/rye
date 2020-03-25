fn ignore_inner_items(_: &mut Context<'_>) {
    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}
