#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const case_sync_nested: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn case_sync_nested(ctx: &mut Context<'_>) {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        __rye::section!(ctx, 0u64, "resizing bigger changes size and capacity", {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 10);

            __rye::section!(ctx, 1u64, "shrinking smaller does not changes capacity", {
                vec.resize(0, 0);
                assert_eq!(vec.len(), 0);
                assert!(vec.capacity() >= 10);
            });
        });
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(case_sync_nested),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking case_sync_nested)
        }

        fn test_plans(&self) -> &'static [__rye::TestPlan] {
            &[
                __rye::TestPlan { target: Some(1u64), ancestors: &[ 0u64 ], },
            ]
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(case_sync_nested);
