#[cfg(any(test, trybuild))]
#[allow(non_camel_case_types)]
struct multi_section_in_scope(());

#[cfg(any(test, trybuild))]
#[allow(non_upper_case_globals)]
const __SCOPE_FOR__multi_section_in_scope: () = {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    impl multi_section_in_scope {
        const fn __new() -> Self {
            Self(())
        }

        fn __body() {
            __rye::enter_section!(0u64, {
                assert!(1 + 1 == 2);
            });

            __rye::enter_section!(1u64, {
                assert!(1 + 1 == 2);

                __rye::enter_section!(2u64, {
                    assert!(true);

                    __rye::enter_section!(3u64, {
                        assert!(true);
                    });
                });

                __rye::enter_section!(4u64, {
                    assert!(true);
                });

                assert!(1 + 2 == 3);
            });

            __rye::enter_section!(5u64, {
                assert!(false);
            });
        }
    }

    impl __rye::TestCase for multi_section_in_scope {
        fn desc(&self) -> __rye::TestDesc {
            __rye::TestDesc {
                name: __rye::test_name!(multi_section_in_scope),
                location: __rye::location!(),
                sections: __rye::sections! {
                    0u64 => ("section1"     , {});
                    1u64 => ("section2"     , {});
                    2u64 => ("section2-1"   , { 1u64 });
                    3u64 => ("section2-1-2" , { 1u64, 2u64 });
                    4u64 => ("section2-2"   , { 1u64 });
                    5u64 => ("section3"     , {});
                },
                leaf_sections: &[ 0u64, 3u64, 4u64, 5u64 ],
            }
        }

        fn test_fn(&self) -> __rye::TestFn {
            __rye::blocking_test_fn!(Self::__body)
        }
    }
};

#[cfg(any(test, trybuild))]
::rye::_internal::register_test_case!(multi_section_in_scope);
