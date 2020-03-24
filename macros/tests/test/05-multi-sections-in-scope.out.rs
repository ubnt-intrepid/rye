#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const multi_section_in_scope: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn multi_section_in_scope() {
        #[allow(unused_imports)]
        use __rye::prelude::*;

        __rye::enter_section!(0u64, "section1", {
            assert!(1 + 1 == 2);
        });

        __rye::enter_section!(1u64, "section2", {
            assert!(1 + 1 == 2);

            __rye::enter_section!(2u64, "section2-1", {
                assert!(true);

                __rye::enter_section!(3u64, "section2-1-2", {
                    assert!(true);
                });
            });

            __rye::enter_section!(4u64, "section2-2", {
                assert!(true);
            });

            assert!(1 + 2 == 3);
        });

        __rye::enter_section!(5u64, "section3", {
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
::rye::_internal::register_test_case!(multi_section_in_scope);
