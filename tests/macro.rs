#[rye::test_case]
fn case_sync() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    rye::section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

#[rye::test_case(block_on = "futures_executor::block_on")]
async fn case_async() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    rye::section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}
