fn case_sync_nested() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    ::rye::__enter_section!(0u64, {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);

        ::rye::__enter_section!(1u64, {
            vec.resize(0, 0);
            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() >= 10);
        });
    });
}

::rye::__declare_test_module! {
    name = case_sync_nested;
    sections = {
        0u64 => ("resizing bigger changes size and capacity", {});
        1u64 => ("shrinking smaller does not changes capacity", { 0u64 });
    };
    leaf_sections = { 1u64 };
    [blocking] test_fn = case_sync_nested;
}
