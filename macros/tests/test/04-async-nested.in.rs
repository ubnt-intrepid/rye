async fn case_async_nested() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);

        section!("shrinking smaller does not changes capacity", {
            vec.resize(0, 0);
            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() >= 10);
        });
    });
}
