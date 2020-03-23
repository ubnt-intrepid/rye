#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const return_result: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn return_result() -> std::io::Result<()>
    where
        std::io::Result<()>: __rye::Termination
    {
        #[allow(unused_imports)]
        use __rye::prelude::*;

        Ok(())
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(return_result),
                location: __rye::location!(),
                sections: __rye::sections! {},
                leaf_sections: &[],
            }
        }
        fn test_fn(&self) -> __rye::TestFn {
            __rye :: test_fn ! ( @ blocking return_result )
        }
    }
    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(return_result);
