fn ignore_inner_items() {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}

pub(crate) mod ignore_inner_items {
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
                    sections: __rye::declare_section! {},
                    leaf_sections: &[],
                },
                __rye::blocking_test_fn!(ignore_inner_items)
            )?;
            __rye::Result::Ok(())
        }
    }

    __rye::cfg_frameworks! {
        #[test_case]
        static __TESTS: &dyn __rye::TestSet = &__tests::new();
    }
}
