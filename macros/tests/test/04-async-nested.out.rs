#[allow(non_camel_case_types)]
struct case_async_nested(());

#[allow(non_upper_case_globals)]
const __SCOPE_FOR__case_async_nested: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl case_async_nested {
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
                assert!(vec.capacity() >= 10);

                __rye::enter_section!(1u64, {
                    vec.resize(0, 0);
                    assert_eq!(vec.len(), 0);
                    assert!(vec.capacity() >= 10);
                });
            });
        }
    }

    impl __rye::TestSet for case_async_nested {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    name: __rye::test_name!(case_async_nested),
                    location: __rye::location!(),
                    sections: __rye::declare_section! {
                        0u64 => ("resizing bigger changes size and capacity", {});
                        1u64 => ("shrinking smaller does not changes capacity", { 0u64 });
                    },
                    leaf_sections: &[ 1u64 ],
                },
                __rye::async_test_fn!(Self::__body)
            )?;
            __rye::Result::Ok(())
        }
    }
};

::rye::_internal::cfg_frameworks! {
    #[test_case]
    static __TEST_CASE__case_async_nested: &dyn ::rye::_internal::TestSet = &case_async_nested::__new();
}
