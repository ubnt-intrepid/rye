#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const case_async: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    async fn case_async() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        __rye::enter_section!(0u64, {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        });
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(case_async),
                location: __rye::location!(),
                sections: __rye::sections! {
                    0u64 => ("resizing bigger changes size and capacity", {});
                },
                leaf_sections: &[ 0u64 ],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@async case_async)
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(case_async);
