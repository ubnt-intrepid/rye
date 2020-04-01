fn main() {}

#[rye::test]
fn section_in_closure(cx: &mut rye::Context<'_>) {
    std::convert::identity(|| {
        section!(cx, "section", {});
    });
}
