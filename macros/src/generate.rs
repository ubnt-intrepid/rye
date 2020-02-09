use crate::section::Section;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Ident, ItemFn, Token};

pub(crate) fn generate(item: ItemFn, sections: Vec<Section>) -> TokenStream {
    let attrs = &item.attrs;
    let vis = &item.vis;
    let asyncness = &item.sig.asyncness;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let inner_fn_ident = Ident::new("__inner__", ident.span());

    let run_test = if asyncness.is_some() {
        quote! {
            TEST_CASE.run_async(#inner_fn_ident).await;
        }
    } else {
        quote! {
            TEST_CASE.run(#inner_fn_ident);
        }
    };

    let sections: Punctuated<_, Token![,]> = sections.into_iter().collect();

    quote! {
        #(#attrs)*
        #vis #asyncness #fn_token #ident () {
            #asyncness #fn_token #inner_fn_ident() #output #block
            static TEST_CASE: rye::_internal::TestCase = rye::_internal::TestCase {
                sections: &[ #sections ],
            };
            #run_test
        }
    }
}
