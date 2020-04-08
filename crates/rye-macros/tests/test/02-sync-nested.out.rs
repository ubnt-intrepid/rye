#[allow(non_upper_case_globals)]
const case_sync_nested: & ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

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

    &__rye::TestCase {
        desc: __rye::TestDesc {
            name: __rye::test_name!(case_sync_nested),
            location: __rye::location!(),
        },
        testfn: __rye::test_fn!(@blocking case_sync_nested),
        plans: &[
            __rye::TestPlan { target: Some(1u64), ancestors: &[ 0u64 ], },
        ],
    }
};

::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_case_sync_nested: & ::rye::_test_reexports::TestCase = case_sync_nested;
}
