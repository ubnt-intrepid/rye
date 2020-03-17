fn main() {}

#[rye::test]
fn section_in_async_block() {
    let _ = std::convert::identity(async {
        section!("section", {
            assert!(true);
        });
    });
}
