fn no_sections(suite: &mut rye::TestSuite<'_>) {
    fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }
    let desc = rye::_internal::TestDesc {
        name: "no_sections",
        module_path: module_path!(),
        sections: rye::_internal::hashmap! {},
        leaf_sections: &[],
    };
    suite.register(desc, __inner__);
}
