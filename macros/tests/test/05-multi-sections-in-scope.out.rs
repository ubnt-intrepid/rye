fn multi_section_in_scope() {
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

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

pub(crate) mod multi_section_in_scope {
    use super::*;
    #[allow(unused_imports)]
    use ::rye::_internal as __rye;

    __rye::lazy_static! {
        static ref __DESC: __rye::TestDesc = __rye::TestDesc {
            module_path: __rye::module_path!(),
            location: __rye::location!(),
            todo: false,
            sections: __rye::declare_section! {
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

    impl __rye::TestSet for __tests {
        fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
            __registry.add_test(&*__DESC, __rye::blocking_test_fn!(multi_section_in_scope))?;
            __rye::Result::Ok(())
        }
    }

    __rye::annotate_test_case! {
        pub(crate) static __TESTS: &dyn __rye::TestSet = &__tests(());
    }
}
