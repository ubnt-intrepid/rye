async fn case_async() {
    let mut vec = vec![0usize; 5];
    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    ::rye::__enter_section!(0u64, {
        vec.resize(10, 0);
        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

pub(crate) mod case_async {
    use super::*;

    ::rye::_internal::lazy_static! {
        static ref __DESC: ::rye::_internal::TestDesc = ::rye::_internal::TestDesc {
            module_path: ::rye::_internal::module_path!(),
            todo: false,
            sections: ::rye::__declare_section! {
                0u64 => ("resizing bigger changes size and capacity", {});
            },
            leaf_sections: &[ 0u64 ],
        };
    }

    #[allow(non_camel_case_types)]
    struct __tests(());

    impl ::rye::_internal::TestSet for __tests {
        fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> ::rye::_internal::Result<(), ::rye::_internal::RegistryError> {
            __registry.add_test(::rye::_internal::Test {
                desc: &*__DESC,
                test_fn: ::rye::__test_fn!([async] case_async),
            })?;
            ::rye::_internal::Result::Ok(())
        }
    }

    ::rye::__annotate_test_case! {
        pub(crate) static __TESTS: &dyn ::rye::_internal::TestSet = &__tests(());
    }
}
