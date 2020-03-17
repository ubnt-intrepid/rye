fn case_sync_nested() {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    __rye::enter_section!(0u64, {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);

        __rye::enter_section!(1u64, {
            vec.resize(0, 0);
            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() >= 10);
        });
    });
}

pub(crate) mod case_sync_nested {
    use super::*;
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    #[allow(non_camel_case_types)]
    pub(crate) struct __tests(());

    impl __tests {
        pub(crate) const fn new() -> Self {
            Self(())
        }
    }

    impl __rye::TestSet for __tests {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    module_path: __rye::module_path!(),
                    location: __rye::location!(),
                    sections: __rye::declare_section! {
                        0u64 => ("resizing bigger changes size and capacity", {});
                        1u64 => ("shrinking smaller does not changes capacity", { 0u64 });
                    },
                    leaf_sections: &[ 1u64 ],
                },
                __rye::blocking_test_fn!(case_sync_nested)
            )?;
            __rye::Result::Ok(())
        }
    }

    __rye::cfg_frameworks! {
        #[test_case]
        static __TESTS: &dyn __rye::TestSet = &__tests::new();
    }
}
