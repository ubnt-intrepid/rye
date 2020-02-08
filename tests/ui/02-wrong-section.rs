fn main() {}

#[rye::test_case]
fn one_section() {
    section!();
}

#[rye::test_case]
fn multi_sections() {
    section!();
    section!();
}

#[rye::test_case]
fn nested_sections() {
    section!("a", {
        section!();
    });
}
