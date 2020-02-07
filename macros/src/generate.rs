use crate::args::Args;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn};

pub(crate) fn generate(
    args: Args,
    item: ItemFn,
    current_section_ident: Ident,
    inner_fn_ident: Ident,
) -> TokenStream {
    let section = &current_section_ident;

    let vis = &item.vis;
    let asyncness = &item.sig.asyncness;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let test_attr: syn::Attribute = syn::parse_quote!(#[test]);
    let attrs = Some(&test_attr).into_iter().chain(&item.attrs);

    let body = if let Some(async_token) = asyncness {
        let block_on: syn::Path = match args
            .values
            .iter()
            .find(|value| value.path.is_ident("block_on"))
        {
            Some(value) => match &value.lit {
                syn::Lit::Str(lit_str) => match lit_str.parse() {
                    Ok(path) => path,
                    Err(err) => return err.to_compile_error(),
                },
                lit => {
                    use syn::spanned::Spanned;
                    let span = lit.span();
                    return quote::quote_spanned!(span => const _: () = compile_error!("should be a string literal"););
                }
            },
            None => syn::parse_quote!(block_on),
        };

        quote! {
            #block_on(#async_token {
                #[allow(unused_mut)]
                let mut test_case = rye::_internal::TestCase::new();
                while !test_case.completed() {
                    #inner_fn_ident(test_case.root_section()).await;
                }
            });
        }
    } else {
        quote! {
            #[allow(unused_mut)]
            let mut test_case = rye::_internal::TestCase::new();
            while !test_case.completed() {
                #inner_fn_ident(test_case.root_section());
            }
        }
    };

    quote! {
        #(#attrs)*
        #vis #fn_token #ident () {
            #asyncness #fn_token #inner_fn_ident(__section: rye::_internal::Section) #output {
                #[allow(unused_mut, unused_variables)]
                let mut #section = __section;
                #block
            }
            #body
        }
    }
}
