#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct ignore_inner_items(());

#[cfg(any(test, trybuild))]
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

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(ignore_inner_items);
