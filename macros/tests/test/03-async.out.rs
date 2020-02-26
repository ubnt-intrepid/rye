async fn case_async() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    ::rye::__enter_section!(0u64, {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

::rye::__declare_test_module! {
    name = case_async;
    sections = {
        0u64 => ("resizing bigger changes size and capacity", {});
    };
    leaf_sections = { 0u64 };
    [async(local = false)] test_fn = case_async;
}
