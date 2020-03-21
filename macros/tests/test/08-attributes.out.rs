#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct attributes(());

#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const __SCOPE_FOR__attributes: () = {
    #[allow(unused_imports)]
    use path::to::rye::_internal as __rye;

    impl attributes {
        const fn __new() -> Self {
            Self(())
        }

        #[allow(missing_docs)]
        fn __body() {
            let mut vec = vec![0usize; 5];
            assert_eq!(vec.len(), 5);
            assert!(vec.capacity() >= 5);

            __rye::enter_section!(
                0u64,
                #[allow(unused_variables)]
                {
                    let foo = 10;
                }
            );
        }
    }

    impl __rye::TestSet for attributes {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(
                __rye::TestDesc {
                    name: __rye::test_name!(attributes),
                    location: __rye::location!(),
                    sections: __rye::declare_section! {
                        0u64 => ("with unused variable", {});
                    },
                    leaf_sections: &[ 0u64 ],
                },
                __rye::blocking_test_fn!(Self::__body)
            )?;
            __rye::Result::Ok(())
        }
    }
};

path::to::rye::_internal::cfg_frameworks! {
    #[test_case]
    static __TEST_CASE__attributes: &dyn path::to::rye::_internal::TestSet = &attributes::__new();
}
