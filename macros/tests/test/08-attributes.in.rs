#[rye(crate = "path::to::rye")]
#[rye(todo)]
#[allow(missing_docs)]
fn attributes() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}
