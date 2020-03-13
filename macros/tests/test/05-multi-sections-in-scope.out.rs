fn multi_section_in_scope() {
    ::rye::__enter_section!(0u64, {
        assert!(1 + 1 == 2);
    });

    ::rye::__enter_section!(1u64, {
        assert!(1 + 1 == 2);

        ::rye::__enter_section!(2u64, {
            assert!(true);

            ::rye::__enter_section!(3u64, {
                assert!(true);
            });
        });

        ::rye::__enter_section!(4u64, {
            assert!(true);
        });

        assert!(1 + 2 == 3);
    });

    ::rye::__enter_section!(5u64, {
        assert!(false);
    });
}

pub(crate) mod multi_section_in_scope {
    use super::*;

    ::rye::_internal::lazy_static! {
        static ref __DESC: ::rye::_internal::TestDesc = ::rye::_internal::TestDesc {
            module_path: ::rye::_internal::module_path!(),
            todo: false,
            sections: ::rye::__declare_section! {
                0u64 => ("section1"     , {});
                1u64 => ("section2"     , {});
                2u64 => ("section2-1"   , { 1u64 });
                3u64 => ("section2-1-2" , { 1u64, 2u64 });
                4u64 => ("section2-2"   , { 1u64 });
                5u64 => ("section3"     , {});
            },
            leaf_sections: &[ 0u64, 3u64, 4u64, 5u64 ],
        };
    }

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl ::rye::_internal::TestSet for __tests {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: &*__DESC,
                test_fn: ::rye::__test_fn!([blocking] multi_section_in_scope),
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) static __TESTS: &dyn ::rye::_internal::TestSet = &__tests(());
    }
}
