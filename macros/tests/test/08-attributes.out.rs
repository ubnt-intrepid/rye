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

    impl __rye::TestCase for attributes {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(attributes),
                location: __rye::location!(),
                sections: __rye::sections! {
                    0u64 => ("with unused variable", {});
                },
                leaf_sections: &[ 0u64 ],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::blocking_test_fn!(Self::__body)
        }
    }
};

#[cfg(any(test, trybuild))]
path::to::rye::_internal::register_test_case!(attributes);
