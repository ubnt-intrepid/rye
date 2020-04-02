#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const case_async: &dyn ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    async fn case_async(ctx: &mut Context<'_>) {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        __rye::section!(ctx, 0u64, "resizing bigger changes size and capacity", {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        });
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(case_async),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@async case_async)
        }

        fn test_plans(&self) -> &'static [__rye::TestPlan] {
            &[
                __rye::TestPlan { target: Some(0u64), ancestors: &[], },
            ]
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_case_async: &dyn ::rye::_test_reexports::TestCase = case_async;
}