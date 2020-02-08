fn case_sync() {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if let Some(mut __section) =
            rye::_internal::new_section(0u64, "resizing bigger changes size and capacity")
        {
            __section.scope(|| {
                vec.resize(10, 0);
                assert_eq!(vec.len(), 10);
                assert!(vec.capacity() >= 5);
            });
        }
    }
    rye::_internal::run(__inner__);
}
