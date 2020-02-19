fn ignore_inner_items() {
    fn inner() {
        section!("section1", {
            assert!(1 + 1 == 2);
        });
    }
}

#[doc(hidden)]
pub mod ignore_inner_items {
    use super::*;

    pub struct __registration(());

    impl ::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: ::rye::_internal::TestDesc {
                    module_path: ::rye::_internal::module_path!(),
                    sections: ::rye::_internal::hashmap! {},
                    leaf_sections: ::rye::_internal::vec![],
                },
                test_fn: ::rye::_internal::TestFn::SyncTest(super::ignore_inner_items),
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub const __REGISTRATION: __registration = __registration(());
    }
}
