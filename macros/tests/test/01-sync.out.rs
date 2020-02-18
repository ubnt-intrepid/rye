fn case_sync() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    if ::rye::_internal::is_target(0u64) {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    }
}

#[doc(hidden)]
pub mod case_sync {
    use super::*;

    pub struct __registration(());

    impl ::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut ::rye::_internal::Registry<'_>) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
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
                test_fn: ::rye::_internal::TestFn::SyncTest(super::case_sync),
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub const __REGISTRATION: __registration = __registration(());
    }
}
