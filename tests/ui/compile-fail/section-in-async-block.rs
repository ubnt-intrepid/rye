fn main() {}

#[rye::test]
fn section_in_closure() {
    let _ = std::convert::identity(async {
        section!("section", {
            assert!(true);
        });
    });
}
