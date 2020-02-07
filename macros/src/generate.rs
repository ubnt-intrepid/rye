use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Ident, ItemFn};

pub(crate) fn generate(
    item: ItemFn,
    current_section_ident: Ident,
    inner_fn_ident: Ident,
) -> TokenStream {
    let section = &current_section_ident;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let asyncness = &item.sig.asyncness;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let await_token = asyncness.as_ref().map(|async_token| {
        use syn::spanned::Spanned;
        quote_spanned!(async_token.span() => .await)
    });

    quote! {
        #(#attrs)*
        #vis #asyncness #fn_token #ident () {
            #asyncness #fn_token #inner_fn_ident(__section: rye::_internal::Section) #output {
                #[allow(unused_mut, unused_variables)]
                let mut #section = __section;
                #block
            }
            #[allow(unused_mut)]
            let mut test_case = rye::_internal::TestCase::new();
            while !test_case.completed() {
                #inner_fn_ident(test_case.root_section()) #await_token;
            }
        }
    }
}
