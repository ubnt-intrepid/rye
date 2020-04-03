fn main() {}

#[rye::test]
fn one_section(cx: &mut rye::Context<'_>) {
    section!(cx);
}

#[rye::test]
fn multi_sections(cx: &mut rye::Context<'_>) {
    section!(cx);
    section!(cx);
}

#[rye::test]
fn nested_sections(cx: &mut rye::Context<'_>) {
    section!(cx, "a", {
        section!(cx);
    });
}
