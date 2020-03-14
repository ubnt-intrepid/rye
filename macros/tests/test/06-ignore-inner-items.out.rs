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
                test_fn: __rye::test_fn!([blocking] ignore_inner_items),
            })?;
            __rye::Result::Ok(())
        }
    }

    __rye::annotate_test_case! {
        pub(crate) static __TESTS: &dyn __rye::TestSet = &__tests(());
    }
}
