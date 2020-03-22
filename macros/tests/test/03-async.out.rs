#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct case_async(());

#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const __SCOPE_FOR__case_async: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl case_async {
        const fn __new() -> Self {
            Self(())
        }

        async fn __body() {
            let mut vec = vec![0usize; 5];
            assert_eq!(vec.len(), 5);
            assert!(vec.capacity() >= 5);

            __rye::enter_section!(0u64, {
                vec.resize(10, 0);
                assert_eq!(vec.len(), 10);
                assert!(vec.capacity() >= 5);
            });
        }
    }

    impl __rye::TestSet for case_async {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    name: __rye::test_name!(case_async),
                    location: __rye::location!(),
                    sections: __rye::sections! {
                        0u64 => ("resizing bigger changes size and capacity", {});
                    },
                    leaf_sections: &[ 0u64 ],
                },
                __rye::async_test_fn!(Self::__body)
            )?;
            __rye::Result::Ok(())
        }
    }
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(case_async);
