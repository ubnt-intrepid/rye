#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const attributes: &dyn path::to::rye::_internal::TestCase = {
    #[allow(unused_imports)]
    use path::to::rye::_internal as __rye;

    #[allow(missing_docs)]
    fn attributes() {
        #[allow(unused_imports)]
        use __rye::prelude::*;

        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        __rye::enter_section!(
            0u64,
            #[allow(unused_variables)]
            {
                let foo = 10;
            }
        );
    }

    struct __TestCase;

    impl __rye::TestCase for __TestCase {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(attributes),
                location: __rye::location!(),
                sections: __rye::sections! {
                    0u64 => ("with unused variable", {});
                },
                leaf_sections: &[ 0u64 ],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::test_fn!(@blocking attributes)
        }
    }

    &__TestCase
};

#[cfg(any(test, trybuild))]
path::to::rye::_internal::register_test_case!(attributes);
