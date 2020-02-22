fn main() {}

#[rye::test]
fn nonunit_ret() -> std::io::Result<()> {
    Ok(())
}

#[rye::test]
fn has_input(x: u32) {}

#[rye::test]
fn has_generics<T>() {}
