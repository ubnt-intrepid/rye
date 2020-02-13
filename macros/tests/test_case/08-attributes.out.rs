fn ignored(suite: &mut rye::_internal::TestSuite<'_>) {
    #[allow(unused_variables)]
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    let desc = rye::_internal::TestDesc {
        name: "ignored",
        module_path: module_path!(),
        ignored: true,
        sections: rye::_internal::hashmap! {},
        leaf_sections: &[],
    };
    suite.register(desc, __inner__);
}