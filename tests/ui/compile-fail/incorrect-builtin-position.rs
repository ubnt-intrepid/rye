fn main() {}

#[rye::test]
fn builtin_in_closure(cx: &mut rye::Context<'_>) {
    drop(|| {
        skip!();
    });
}

#[rye::test]
fn builtin_in_async_block(cx: &mut rye::Context<'_>) {
    drop(async {
        skip!();
    });
}
