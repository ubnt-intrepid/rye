fn main() {}

#[rye::test_case]
fn section_in_closure() {
    let _ = std::convert::identity(async {
        section!("section", {
            assert!(true);
        });
    });
}
