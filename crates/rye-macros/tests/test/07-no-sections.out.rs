#[allow(non_upper_case_globals)]
const no_sections: & ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn no_sections(_: &mut Context<'_>) {}

    &__rye::TestCase {
        desc: __rye::TestDesc {
            name: __rye::test_name!(no_sections),
            location: __rye::location!(),
        },
        testfn: __rye::test_fn!(@blocking no_sections),
        plans: &[
            __rye::TestPlan { target: None, ancestors: &[], },
        ],
    }
};

::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_no_sections: & ::rye::_test_reexports::TestCase = no_sections;
}
