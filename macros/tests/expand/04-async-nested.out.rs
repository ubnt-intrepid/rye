async fn case_async_nested() {
    async fn __inner__() {
        let mut vec = vec![0usize; 5];
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        {
            static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                name: "resizing bigger changes size and capacity",
                file: file!(),
                line: line!(),
                column: column!(),
            };
            if let Some(mut __section) = rye::_internal::new_section(&SECTION) {
                __section.scope_async(async {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 10);

                    {
                        static SECTION: rye::_internal::SectionId =
                            rye::_internal::SectionId::SubSection {
                                name: "shrinking smaller does not changes capacity",
                                file: file!(),
                                line: line!(),
                                column: column!(),
                            };
                        if let Some(mut __section) = rye::_internal::new_section(&SECTION) {
                            __section.scope_async(async {
                                vec.resize(0, 0);
                                assert_eq!(vec.len(), 0);
                                assert!(vec.capacity() >= 10);
                            })
                            .await;
                        }
                    }
                })
                .await;
            }
        }
    }
    rye::_internal::run_async(__inner__).await;
}
