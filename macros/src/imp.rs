use crate::args::Args;
use proc_macro2::{Span, TokenStream};
use syn::{parse::Result, Ident, ItemFn};

const CURRENT_SECTION_IDENT_NAME: &str = "__rye_current_section__";
const INNER_FN_IDENT_NAME: &str = "__rye_inner_fn__";

pub(crate) fn test_case(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: Args = syn::parse2(args)?;

    let mut item: ItemFn = syn::parse2(item)?;

    let current_section_ident = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
    let inner_fn_ident = Ident::new(INNER_FN_IDENT_NAME, item.sig.ident.span());

    crate::expand::expand(&mut *item.block, &current_section_ident)?;

    Ok(crate::generate::generate(
        args,
        item,
        current_section_ident,
        inner_fn_ident,
    ))
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

        let section = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
        let inner = Ident::new(INNER_FN_IDENT_NAME, Span::call_site());
        let expected = quote! {
            #[test]
            fn case_sync() {
                fn #inner(__section: rye::_internal::Section) {
                    #[allow(unused_mut, unused_variables)]
                    let mut #section = __section;
                    {
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
                            if let Some(__section) = #section.new_section(&SECTION) {
                                let mut #section = __section;
                                {
                                    vec.resize(10, 0);
                                    assert_eq!(vec.len(), 10);
                                    assert!(vec.capacity() >= 5);
                                }
                            }
                        }
                    }
                }

                #[allow(unused_mut)]
                let mut test_case = rye::_internal::TestCase::new();
                while !test_case.completed() {
                    #inner(test_case.root_section());
                }
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
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

        let section = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
        let inner = Ident::new(INNER_FN_IDENT_NAME, Span::call_site());
        let expected = quote! {
            #[test]
            fn case_sync() {
                fn #inner(__section: rye::_internal::Section) {
                    #[allow(unused_mut, unused_variables)]
                    let mut #section = __section;
                    {
                        let mut vec = vec![0usize; 5];
                        assert_eq!(vec.len(), 5);
                        assert!(vec.capacity() >= 5);

                        {
                            static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                                name: "resizing bigger changes size and capacity" ,
                                file: file!(),
                                line: line!(),
                                column: column!(),
                            };
                            if let Some(__section) = #section.new_section(&SECTION) {
                                let mut #section = __section;
                                {
                                    vec.resize(10, 0);
                                    assert_eq!(vec.len(), 10);
                                    assert!(vec.capacity() >= 10);

                                    {
                                        static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                                            name: "shrinking smaller does not changes capacity" ,
                                            file: file!(),
                                            line: line!(),
                                            column: column!(),
                                        };
                                        if let Some(__section) = #section.new_section(&SECTION) {
                                            let mut #section = __section;
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
                }

                #[allow(unused_mut)]
                let mut test_case = rye::_internal::TestCase::new();
                while !test_case.completed() {
                    #inner(test_case.root_section());
                }
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
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

        let section = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
        let inner = Ident::new(INNER_FN_IDENT_NAME, Span::call_site());
        let expected = quote! {
            #[test]
            fn case_async() {
                async fn #inner(__section: rye::_internal::Section) {
                    #[allow(unused_mut, unused_variables)]
                    let mut #section = __section;
                    {
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
                            if let Some(__section) = #section.new_section(&SECTION) {
                                let mut #section = __section;
                                {
                                    vec.resize(10, 0);
                                    assert_eq!(vec.len(), 10);
                                    assert!(vec.capacity() >= 5);
                                }
                            }
                        }
                    }
                }

                block_on(async {
                    #[allow(unused_mut)]
                    let mut test_case = rye::_internal::TestCase::new();
                    while !test_case.completed() {
                        #inner(test_case.root_section()).await;
                    }
                });
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
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

        let section = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
        let inner = Ident::new(INNER_FN_IDENT_NAME, Span::call_site());
        let expected = quote! {
            #[test]
            fn case_async() {
                async fn #inner(__section: rye::_internal::Section) {
                    #[allow(unused_mut, unused_variables)]
                    let mut #section = __section;
                    {
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
                            if let Some(__section) = #section.new_section(&SECTION) {
                                let mut #section = __section;
                                {
                                    vec.resize(10, 0);
                                    assert_eq!(vec.len(), 10);
                                    assert!(vec.capacity() >= 5);
                                }
                            }
                        }
                    }
                }

                path::to::custom_block_on(async {
                    #[allow(unused_mut)]
                    let mut test_case = rye::_internal::TestCase::new();

                    while !test_case.completed() {
                        #inner(test_case.root_section()).await;
                    }
                });
            }
        };

        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
    }
}
