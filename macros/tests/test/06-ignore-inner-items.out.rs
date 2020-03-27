#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const ignore_inner_items: &dyn ::rye::_test_reexports::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_test_reexports as __rye;

    fn ignore_inner_items(_: &mut Context<'_>) {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> &'static __rye::TestDesc {
            &__rye::TestDesc {
                name: __rye::test_name!(ignore_inner_items),
                location: __rye::location!(),
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking ignore_inner_items)
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
::rye::_test_reexports::register_test_case!(ignore_inner_items);
