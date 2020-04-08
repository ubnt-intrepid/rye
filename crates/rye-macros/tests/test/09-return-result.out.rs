#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const return_result: & ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn return_result(_: &mut Context<'_>) -> std::io::Result<()>
    where
        std::io::Result<()>: __rye::Termination
    {
        Ok(())
    }

    &__rye::TestCase {
        desc: __rye::TestDesc {
            name: __rye::test_name!(return_result),
            location: __rye::location!(),
        },
        testfn: __rye::test_fn!(@blocking return_result),
        plans: &[
            __rye::TestPlan { target: None, ancestors: &[], },
        ],
    }
};

#[cfg(any(test, trybuild))]
::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_return_result: & ::rye::_test_reexports::TestCase = return_result;
}
