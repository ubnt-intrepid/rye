async fn case_async() {
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
    }
}

pub(crate) mod case_async {
    use super::*;

    struct __registration(());

    impl ::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: ::rye::_internal::TestDesc {
                    module_path: ::rye::_internal::module_path!(),
                    sections: ::rye::_internal::hashmap! {
                        0u64 => ::rye::_internal::Section {
                            name: "resizing bigger changes size and capacity",
                            ancestors: ::rye::_internal::hashset!(),
                        },
                    },
                    leaf_sections: ::rye::_internal::vec![ 0u64 ],
                },
                test_fn: ::rye::_internal::TestFn::AsyncTest {
                    f: || ::rye::_internal::TestFuture::new(super::case_async()),
                    local: false,
                },
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) const __REGISTRATION: &dyn ::rye::_internal::Registration = &__registration(());
    }
}
