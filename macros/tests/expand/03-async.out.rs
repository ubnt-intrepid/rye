fn case_async(suite: &mut rye::TestSuite<'_>) {
    async fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        if rye::_internal::is_target(0u64) {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        }
    }
    let desc = rye::_internal::TestDesc {
        name: "case_async",
        module_path: module_path!(),
        ignored: false,
        sections: rye::_internal::hashmap! {
            0u64 => rye::_internal::Section::new("resizing bigger changes size and capacity", rye::_internal::hashset!()),
        },
        leaf_sections: &[ 0u64 ],
    };
    suite.register_async(desc, __inner__);
}
