fn multi_section_in_scope() {
    {
        let __section = ::rye::_internal::enter_section(0u64);
        if __section.enabled() {
            assert!(1 + 1 == 2);
        }
        __section.leave();
    }

    {
        let __section = ::rye::_internal::enter_section(1u64);
        if __section.enabled() {
            assert!(1 + 1 == 2);

            {
                let __section = ::rye::_internal::enter_section(2u64);
                if __section.enabled() {
                    assert!(true);

                    {
                        let __section = ::rye::_internal::enter_section(3u64);
                        if __section.enabled() {
                            assert!(true);
                        }
                        __section.leave();
                    }
                }
                __section.leave();
            }

            {
                let __section = ::rye::_internal::enter_section(4u64);
                if __section.enabled() {
                    assert!(true);
                }
                __section.leave();
            }

            assert!(1 + 2 == 3);
        }
        __section.leave();
    }

    {
        let __section = ::rye::_internal::enter_section(5u64);
        if __section.enabled() {
            assert!(false);
        }
        __section.leave();
    }
}

pub(crate) mod multi_section_in_scope {
    use super::*;

    ::rye::_internal::lazy_static! {
        static ref DESC: ::rye::_internal::TestDesc = ::rye::_internal::TestDesc {
            module_path: ::rye::_internal::module_path!(),
            sections: ::rye::_internal::hashmap! {
                0u64 => ::rye::_internal::Section { name: "section1"     , ancestors: ::rye::_internal::hashset!()           , },
                1u64 => ::rye::_internal::Section { name: "section2"     , ancestors: ::rye::_internal::hashset!()           , },
                2u64 => ::rye::_internal::Section { name: "section2-1"   , ancestors: ::rye::_internal::hashset!(1u64)       , },
                3u64 => ::rye::_internal::Section { name: "section2-1-2" , ancestors: ::rye::_internal::hashset!(1u64, 2u64) , },
                4u64 => ::rye::_internal::Section { name: "section2-2"   , ancestors: ::rye::_internal::hashset!(1u64)       , },
                5u64 => ::rye::_internal::Section { name: "section3"     , ancestors: ::rye::_internal::hashset!()           , },
            },
            leaf_sections: ::rye::_internal::vec![ 0u64, 3u64, 4u64, 5u64 ],
        };
    }

    struct __registration(());

    impl ::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: &*DESC,
                test_fn: ::rye::_internal::TestFn::Blocking {
                    f: || ::rye::_internal::test_result(super::multi_section_in_scope()),
                },
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) const __REGISTRATION: &dyn ::rye::_internal::Registration = &__registration(());
    }
}
