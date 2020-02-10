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

    let register = if asyncness.is_some() {
        quote! {
            suite.register_async(&TEST_DESC, #inner_fn_ident);
        }
    } else {
        quote! {
            suite.register(&TEST_DESC, #inner_fn_ident);
        }
    };

    let test_name = ident.to_string();

    let sections: Punctuated<_, Token![,]> = sections.into_iter().collect();

    quote! {
        #(#attrs)*
        #vis #fn_token #ident (suite: &mut rye::TestSuite<'_>) {
            #asyncness #fn_token #inner_fn_ident() #output #block
            static TEST_DESC: rye::_internal::TestDesc = rye::_internal::TestDesc {
                name: #test_name,
                module_path: module_path!(),
                sections: &[ #sections ],
            };
            #register
        }
    }
}
