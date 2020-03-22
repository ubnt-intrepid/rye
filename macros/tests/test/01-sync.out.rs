#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct case_sync(());

#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const __SCOPE_FOR__case_sync: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl case_sync {
        const fn __new() -> Self {
            Self(())
        }

        fn __body() {
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

    impl __rye::TestCase for case_sync {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(case_sync),
                location: __rye::location!(),
                sections: __rye::sections! {
                    0u64 => ("resizing bigger changes size and capacity", {});
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
::rye::_internal::register_test_case!(case_sync);
