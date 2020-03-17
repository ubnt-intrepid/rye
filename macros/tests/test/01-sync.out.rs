fn case_sync() {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    __rye::enter_section!(0u64, {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

pub(crate) mod case_sync {
    use super::*;
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl __rye::TestSet for __tests {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    module_path: __rye::module_path!(),
                    location: __rye::location!(),
                    sections: __rye::declare_section! {
                        0u64 => ("resizing bigger changes size and capacity", {});
                    },
                    leaf_sections: &[ 0u64 ],
                },
                __rye::blocking_test_fn!(case_sync)
            )?;
            __rye::Result::Ok(())
        }
    }

    __rye::annotate_test_case! {
        pub(crate) static __TESTS: &dyn __rye::TestSet = &__tests(());
    }
}
