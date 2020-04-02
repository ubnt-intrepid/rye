#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const return_result: &dyn ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn return_result(_: &mut Context<'_>) -> std::io::Result<()>
    where
        std::io::Result<()>: __rye::Termination
    {
        Ok(())
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(return_result),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye :: test_fn ! ( @ blocking return_result )
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
::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_return_result: &dyn ::rye::_test_reexports::TestCase = return_result;
}
