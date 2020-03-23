#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const ignore_inner_items: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn ignore_inner_items() {
        fn inner() {
            section!("section1", {
                assert!(1 + 1 == 2);
            });
        }
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(ignore_inner_items),
                location: __rye::location!(),
                sections: __rye::sections! {},
                leaf_sections: &[],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking ignore_inner_items)
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(ignore_inner_items);
