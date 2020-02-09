fn main() {}

#[rye::test_case]
fn section_in_closure() {
    std::convert::identity(|| {
        section!("section", {
            assert!(true);
        });
    });
}
