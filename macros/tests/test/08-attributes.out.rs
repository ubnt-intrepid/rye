#[allow(missing_docs)]
fn attributes() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}

pub(crate) mod attributes {
    use super::*;

    path::to::rye::_internal::lazy_static! {
        static ref DESC: path::to::rye::_internal::TestDesc = path::to::rye::_internal::TestDesc {
            module_path: path::to::rye::_internal::module_path!(),
            sections: path::to::rye::_internal::hashmap! {},
            leaf_sections: path::to::rye::_internal::vec![],
        };
    }

    struct __registration(());

    impl path::to::rye::_internal::Registration for __registration {
        fn register(&self, __registry: &mut dyn path::to::rye::_internal::Registry) -> path::to::rye::_internal::Result<(), path::to::rye::_internal::RegistryError> {
            __registry.add_test(path::to::rye::_internal::Test {
                desc: &*DESC,
                test_fn: path::to::rye::_internal::TestFn::Blocking {
                    f: || path::to::rye::_internal::test_result(super::attributes()),
                },
            })?;
            path::to::rye::_internal::Result::Ok(())
        }
    }

    path::to::rye::__annotate_test_case! {
        pub(crate) const __REGISTRATION: &dyn path::to::rye::_internal::Registration = &__registration(());
    }
}
