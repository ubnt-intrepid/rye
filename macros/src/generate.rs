use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn};

pub(crate) fn generate(item: ItemFn) -> TokenStream {
    let attrs = &item.attrs;
    let vis = &item.vis;
    let asyncness = &item.sig.asyncness;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let inner_fn_ident = Ident::new("__inner__", ident.span());

    let scoped = if asyncness.is_some() {
        quote! {
            rye::_internal::run_async(#inner_fn_ident).await;
        }
    } else {
        quote! {
            rye::_internal::run(#inner_fn_ident);
        }
    };

    quote! {
        #(#attrs)*
        #vis #asyncness #fn_token #ident () {
            #asyncness #fn_token #inner_fn_ident() #output #block
            #scoped
        }
    }
}
