#[allow(missing_docs)]
fn attributes() {
    #[allow(unused_imports)]
    use path::to::rye::_internal as __rye;

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

pub(crate) mod attributes {
    use super::*;
    #[allow(unused_imports)]
    use path::to::rye::_internal as __rye;

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
                        0u64 => ("with unused variable", {});
                    },
                    leaf_sections: &[ 0u64 ],
                },
                __rye::blocking_test_fn!(attributes)
            )?;
            __rye::Result::Ok(())
        }
    }

    __rye::cfg_frameworks! {
        #[test_case]
        static __TESTS: &dyn __rye::TestSet = &__tests::new();
    }
}
