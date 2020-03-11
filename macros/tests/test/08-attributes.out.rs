#[allow(missing_docs)]
fn attributes() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);
}

pub(crate) mod attributes {
    use super::*;

    path::to::rye::_internal::lazy_static! {
        static ref __DESC: path::to::rye::_internal::TestDesc = path::to::rye::_internal::TestDesc {
            module_path: path::to::rye::_internal::module_path!(),
            sections: path::to::rye::__declare_section! {},
            leaf_sections: &[],
        };
    }

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl path::to::rye::_internal::TestSet for __tests {
        fn register(&self, __registry: &mut dyn path::to::rye::_internal::Registry) -> path::to::rye::_internal::Result<(), path::to::rye::_internal::RegistryError> {
            __registry.add_test(path::to::rye::_internal::Test {
                desc: &*__DESC,
                test_fn: path::to::rye::__test_fn!([blocking] attributes),
            })?;
            path::to::rye::_internal::Result::Ok(())
        }
    }

    path::to::rye::__annotate_test_case! {
        pub(crate) static __TESTS: &dyn path::to::rye::_internal::TestSet = &__tests(());
    }
}
