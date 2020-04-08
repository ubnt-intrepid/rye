#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const attributes: & path::to::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use path::to::rye::_test_reexports as __rye;

    #[allow(missing_docs)]
    fn attributes(ctx: &mut Context<'_>) {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        __rye::section!(
            ctx,
            0u64,
            "with unused variable",
            #[allow(unused_variables)]
            {
                let foo = 10;
            }
        );
    }

    &__rye::TestCase {
        desc: __rye::TestDesc {
            name: __rye::test_name!(attributes),
            location: __rye::location!(),
        },
        testfn: __rye::test_fn!(@blocking attributes),
        plans: &[
            __rye::TestPlan { target: Some(0u64), ancestors: &[], },
        ],
    }
};

#[cfg(any(test, trybuild))]
path::to::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_attributes: & path::to::rye::_test_reexports::TestCase = attributes;
}
