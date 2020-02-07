fn main() {}

fn custom_block_on<F>(fut: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
{
    futures_executor::block_on(fut)
}

#[rye::test_case(block_on = "custom_block_on")]
async fn nested() {
    use futures_test::future::FutureTestExt;

    let mut vec = vec![0usize; 5];
    let rc = std::rc::Rc::new(());

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        async {}.pending_once().await;

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);

        section!("shrinking smaller does not changes capacity", {
            vec.resize(0, 0);

            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() >= 10);
        });
    });

    drop(rc);
}

#[rye::test_case(block_on = "futures_executor::block_on")]
async fn case_async() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}
