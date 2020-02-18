fn main() {}

#[rye::test]
fn one_section() {
    section!();
}

#[rye::test]
fn multi_sections() {
    section!();
    section!();
}

#[rye::test]
fn nested_sections() {
    section!("a", {
        section!();
    });
}
