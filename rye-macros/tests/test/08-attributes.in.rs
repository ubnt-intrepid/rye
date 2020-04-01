#[rye(crate = path::to::rye)]
#[allow(missing_docs)]
fn attributes(ctx: &mut Context<'_>) {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    #[allow(unused_variables)]
    section!(ctx, "with unused variable", {
        let foo = 10;
    });
}
