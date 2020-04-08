#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const ignore_inner_items: & ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn ignore_inner_items(_: &mut Context<'_>) {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }

    &__rye::TestCase {
        desc: __rye::TestDesc {
            name: __rye::test_name!(ignore_inner_items),
            location: __rye::location!(),
        },
        testfn: __rye::test_fn!(@blocking ignore_inner_items),
        plans: &[
            __rye::TestPlan { target: None, ancestors: &[], },
        ],
    }
};

#[cfg(any(test, trybuild))]
::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_ignore_inner_items: & ::rye::_test_reexports::TestCase = ignore_inner_items;
}
