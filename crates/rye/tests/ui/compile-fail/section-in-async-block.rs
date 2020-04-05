fn main() {}

#[rye::test]
fn section_in_async_block(cx: &mut rye::Context<'_>) {
    let _ = std::convert::identity(async {
        section!(cx, "section", {});
    });
}
