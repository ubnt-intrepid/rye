fn ignore_inner_items() {
    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}
