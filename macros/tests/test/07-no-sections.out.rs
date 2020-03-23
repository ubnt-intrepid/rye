#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const no_sections: &dyn ::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    fn no_sections() {
        #[allow(unused_imports)]
        use __rye::prelude::*;

        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(no_sections),
                location: __rye::location!(),
                sections: __rye::sections! {},
                leaf_sections: &[],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking no_sections)
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(no_sections);
