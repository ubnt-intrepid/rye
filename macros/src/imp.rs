use crate::args::Args;
use proc_macro2::TokenStream;
use syn::{parse::Result, ItemFn};

pub(crate) fn test_case(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: Args = syn::parse2(args)?;

    let mut item: ItemFn = syn::parse2(item)?;

    crate::expand::expand(&mut *item.block)?;

    Ok(crate::generate::generate(args, item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_sync() {
        let args = TokenStream::new();
        let item = quote! {
            fn case_sync() {
                let mut vec = vec![0usize; 5];
                assert_eq!(vec.len(), 5);
                assert!(vec.capacity() >= 5);

                section!("resizing bigger changes size and capacity", {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 5);
                });
            }
        };
        let expected = quote! {
            #[test]
            fn case_sync() {
                fn __inner__() {
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

                        if let Some(section) = rye::_internal::new_section(&SECTION) {
                            let _guard = rye::_internal::Guard::set(Some(Box::new(section)));

                            {
                                vec.resize(10, 0);
                                assert_eq!(vec.len(), 10);
                                assert!(vec.capacity() >= 5);
                            }
                        }
                    }
                }

                let mut test_case = rye::TestCase::new();
                while !test_case.completed() {
                    test_case.run(__inner__);
                }
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_nested() {
        let args = TokenStream::new();
        let item = quote! {
            fn case_sync() {
                let mut vec = vec![0usize; 5];
                assert_eq!(vec.len(), 5);
                assert!(vec.capacity() >= 5);

                section!("resizing bigger changes size and capacity", {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 10);

                    section!("shrinking smaller does not changes capacity", {
                        vec.resize(0, 0);
                        assert_eq!(vec.len(), 0);
                        assert!(vec.capacity() >= 10);
                    });
                });
            }
        };
        let expected = quote! {
            #[test]
            fn case_sync() {
                fn __inner__() {
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

                        if let Some(section) = rye::_internal::new_section(&SECTION) {
                            let _guard = rye::_internal::Guard::set(Some(Box::new(section)));

                            {
                                vec.resize(10, 0);
                                assert_eq!(vec.len(), 10);
                                assert!(vec.capacity() >= 10);

                                {
                                    static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                                        name: "shrinking smaller does not changes capacity",
                                        file: file!(),
                                        line: line!(),
                                        column: column!(),
                                    };

                                    if let Some(section) = rye::_internal::new_section(&SECTION) {
                                        let _guard = rye::_internal::Guard::set(Some(Box::new(section)));

                                        {
                                            vec.resize(0, 0);
                                            assert_eq!(vec.len(), 0);
                                            assert!(vec.capacity() >= 10);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let mut test_case = rye::TestCase::new();
                while !test_case.completed() {
                    test_case.run(__inner__);
                }
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_async() {
        let args = TokenStream::new();
        let item = quote! {
            async fn case_async() {
                let mut vec = vec![0usize; 5];
                assert_eq!(vec.len(), 5);
                assert!(vec.capacity() >= 5);

                section!("resizing bigger changes size and capacity", {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 5);
                });
            }
        };
        let expected = quote! {
            #[test]
            fn case_async() {
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

                        if let Some(section) = rye::_internal::new_section(&SECTION) {
                            let _guard = rye::_internal::Guard::set(Some(Box::new(section)));

                            {
                                vec.resize(10, 0);
                                assert_eq!(vec.len(), 10);
                                assert!(vec.capacity() >= 5);
                            }
                        }
                    }
                }

                block_on(async {
                    let mut test_case = rye::TestCase::new();
                    while !test_case.completed() {
                        test_case.run_async(__inner__()).await;
                    }
                });
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_async_with_args() {
        let args = quote!(block_on = "path::to::custom_block_on");
        let item = quote! {
            async fn case_async() {
                let mut vec = vec![0usize; 5];
                assert_eq!(vec.len(), 5);
                assert!(vec.capacity() >= 5);

                section!("resizing bigger changes size and capacity", {
                    vec.resize(10, 0);
                    assert_eq!(vec.len(), 10);
                    assert!(vec.capacity() >= 5);
                });
            }
        };
        let expected = quote! {
            #[test]
            fn case_async() {
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

                        if let Some(section) = rye::_internal::new_section(&SECTION) {
                            let _guard = rye::_internal::Guard::set(Some(Box::new(section)));

                            {
                                vec.resize(10, 0);
                                assert_eq!(vec.len(), 10);
                                assert!(vec.capacity() >= 5);
                            }
                        }
                    }
                }

                path::to::custom_block_on(async {
                    let mut test_case = rye::TestCase::new();
                    while !test_case.completed() {
                        test_case.run_async(__inner__()).await;
                    }
                });
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(output.to_string(), expected.to_string());
    }
}
