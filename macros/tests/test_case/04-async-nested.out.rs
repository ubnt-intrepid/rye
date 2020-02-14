fn case_async_nested(suite: &mut ::rye::_internal::TestSuite<'_>) {
    async fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if ::rye::_internal::is_target(0u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 10);

            if ::rye::_internal::is_target(1u64) {
                vec.resize(0, 0);
                assert_eq!(vec.len(), 0);
                assert!(vec.capacity() >= 10);
            }
        }
    }
    let desc = ::rye::_internal::TestDesc {
        name: "case_async_nested",
        module_path: module_path!(),
        ignored: false,
        sections: ::rye::_internal::hashmap! {
            0u64 => ::rye::_internal::Section::new("resizing bigger changes size and capacity", ::rye::_internal::hashset!()),
            1u64 => ::rye::_internal::Section::new("shrinking smaller does not changes capacity", ::rye::_internal::hashset!(0u64)),
        },
        leaf_sections: &[ 1u64 ],
    };
    fn __async_inner__() -> ::rye::_internal::BoxFuture<'static, ()> {
        Box::pin(__inner__())
    }
    suite.register_async(desc, __async_inner__);
}
