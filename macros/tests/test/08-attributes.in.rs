#[rye(crate = path::to::rye)]
#[allow(missing_docs)]
fn attributes() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    #[allow(unused_variables)]
    section!("with unused variable", {
        let foo = 10;
    });
}
