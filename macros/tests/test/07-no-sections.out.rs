fn no_sections() {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}

pub(crate) mod no_sections {
    use super::*;
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    __rye::lazy_static! {
        static ref __DESC: __rye::TestDesc = __rye::TestDesc {
            module_path: __rye::module_path!(),
            todo: false,
            sections: __rye::declare_section! {},
            leaf_sections: &[],
        };
    }

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl __rye::TestSet for __tests {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(__rye::Test {
                desc: &*__DESC,
                test_fn: __rye::test_fn!([blocking] no_sections),
            })?;
            __rye::Result::Ok(())
        }
    }

    __rye::annotate_test_case! {
        pub(crate) static __TESTS: &dyn __rye::TestSet = &__tests(());
    }
}
