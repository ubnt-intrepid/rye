fn case_sync() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    {
        let __section = ::rye::_internal::enter_section(0u64);
        if __section.enabled() {
            vec.resize(10, 0);
            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        }
        __section.leave();
    }
}

pub(crate) mod case_sync {
    use super::*;

    ::rye::_internal::lazy_static! {
        static ref DESC: ::rye::_internal::TestDesc = ::rye::_internal::TestDesc {
            module_path: ::rye::_internal::module_path!(),
            sections: ::rye::_internal::hashmap! {
                0u64 => ::rye::_internal::Section {
                    name: "resizing bigger changes size and capacity",
                    ancestors: ::rye::_internal::hashset!(),
                },
            },
            leaf_sections: ::rye::_internal::vec![ 0u64 ],
        };
    }

    struct __registration(());

    impl ::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: &*DESC,
                test_fn: ::rye::_internal::TestFn::Blocking {
                    f: || ::rye::_internal::test_result(super::case_sync()),
                },
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) const __REGISTRATION: &dyn ::rye::_internal::Registration = &__registration(());
    }
}
