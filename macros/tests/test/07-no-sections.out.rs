#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct no_sections(());

#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const __SCOPE_FOR__no_sections: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl no_sections {
        const fn __new() -> Self {
            Self(())
        }

        fn __body() {
            let mut vec = vec![0usize; 5];
            assert_eq!(vec.len(), 5);
            assert!(vec.capacity() >= 5);
        }
    }

    impl __rye::TestSet for no_sections {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    name: __rye::test_name!(no_sections),
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
    static __TEST_CASE__no_sections: &dyn ::rye::_internal::TestSet = &no_sections::__new();
}
