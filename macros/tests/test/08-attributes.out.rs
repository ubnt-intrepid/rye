#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const attributes: &dyn path::to::rye::_test_reexports::TestCase = {
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

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(attributes),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking attributes)
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
path::to::rye::_test_reexports::register_test_case!(attributes);
