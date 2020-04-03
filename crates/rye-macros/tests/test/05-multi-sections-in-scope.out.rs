#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const multi_section_in_scope: &dyn ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn multi_section_in_scope(ctx: &mut Context<'_>) {
        __rye::section!(ctx, 0u64, "section1", {
            assert!(1 + 1 == 2);
        });

        __rye::section!(ctx, 1u64, "section2", {
            assert!(1 + 1 == 2);

            __rye::section!(ctx, 2u64, "section2-1", {
                assert!(true);

                __rye::section!(ctx, 3u64, "section2-1-2", {
                    assert!(true);
                });
            });

            __rye::section!(ctx, 4u64, "section2-2", {
                assert!(true);
            });

            assert!(1 + 2 == 3);
        });

        __rye::section!(ctx, 5u64, "section3", {
            assert!(false);
        });
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(multi_section_in_scope),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking multi_section_in_scope)
        }

        fn test_plans(&self) -> &'static [__rye::TestPlan] {
            &[
                __rye::TestPlan { target: Some(0u64), ancestors: &[], },
                __rye::TestPlan { target: Some(3u64), ancestors: &[ 1u64, 2u64 ], },
                __rye::TestPlan { target: Some(4u64), ancestors: &[ 1u64 ], },
                __rye::TestPlan { target: Some(5u64), ancestors: &[], },
            ]
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::__test_case! {
    #[allow(non_upper_case_globals)]
    static __TEST_CASE_multi_section_in_scope: &dyn ::rye::_test_reexports::TestCase = multi_section_in_scope;
}
