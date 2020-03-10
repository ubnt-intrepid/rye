fn main() {}

#[rye::test]
fn section_in_closure() {
    std::convert::identity(|| {
        section!("section", {
            assert!(true);
        });
    });
}
