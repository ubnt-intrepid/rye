async fn case_async_nested() {
    async fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if let Some(mut __section) =
            rye::_internal::new_section(0u64, "resizing bigger changes size and capacity")
        {
            __section
                .scope_async(async {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 10);

                    if let Some(mut __section) = rye::_internal::new_section(
                        1u64,
                        "shrinking smaller does not changes capacity"
                    ) {
                        __section
                            .scope_async(async {
                                vec.resize(0, 0);
                                assert_eq!(vec.len(), 0);
                                assert!(vec.capacity() >= 10);
                            })
                            .await;
                    }
                })
                .await;
        }
    }
    rye::_internal::run_async(__inner__).await;
}
