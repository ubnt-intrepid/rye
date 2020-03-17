#[allow(non_camel_case_types)]
struct ignore_inner_items(());

#[allow(non_upper_case_globals)]
const __SCOPE_FOR__ignore_inner_items: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl ignore_inner_items {
        const fn __new() -> Self {
            Self(())
        }

        fn __body() {
            fn inner() {
                section!("section1", {
                    assert!(1 + 1 == 2);
                });
            }
        }
    }

    impl __rye::TestSet for ignore_inner_items {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    name: __rye::test_name!(ignore_inner_items),
                    location: __rye::location!(),
                    sections: __rye::declare_section! {},
                    leaf_sections: &[],
                },
                __rye::blocking_test_fn!(Self::__body)
            )?;
            __rye::Result::Ok(())
        }
    }
};

::rye::_internal::cfg_frameworks! {
    #[test_case]
    static __TEST_CASE__ignore_inner_items: &dyn ::rye::_internal::TestSet = &ignore_inner_items::__new();
}
