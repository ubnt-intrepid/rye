#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const no_sections: &dyn ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn no_sections(_: &mut Context<'_>) {}

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(no_sections),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking no_sections)
        }

        fn test_plans(&self) -> &'static [__rye::TestPlan] {
            &[
                __rye::TestPlan { target: None, ancestors: &[], },
            ]
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_test_reexports::test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_no_sections: &dyn ::rye::_test_reexports::TestCase = no_sections;
}
