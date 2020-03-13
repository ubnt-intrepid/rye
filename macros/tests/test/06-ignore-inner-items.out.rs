fn ignore_inner_items() {
    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}

pub(crate) mod ignore_inner_items {
    use super::*;

    ::rye::_internal::lazy_static! {
        static ref __DESC: ::rye::_internal::TestDesc = ::rye::_internal::TestDesc {
            module_path: ::rye::_internal::module_path!(),
            todo: false,
            sections: ::rye::__declare_section! {},
            leaf_sections: &[],
        };
    }

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl ::rye::_internal::TestSet for __tests {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: &*__DESC,
                test_fn: ::rye::__test_fn!([blocking] ignore_inner_items),
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) static __TESTS: &dyn ::rye::_internal::TestSet = &__tests(());
    }
}
