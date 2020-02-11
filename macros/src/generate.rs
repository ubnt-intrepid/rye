use crate::section::Section;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn};

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
            suite.register_async(desc, #inner_fn_ident);
        }
    } else {
        quote! {
            suite.register(desc, #inner_fn_ident);
        }
    };

    let test_name = ident.to_string();

    let sections = &sections;

    let section_map_entries = sections.iter().map(|section| section.map_entry());

    let leaf_section_ids = sections.iter().filter_map(|section| {
        if section.children.is_empty() {
            Some(section.id)
        } else {
            None
        }
    });

    quote! {
        #(#attrs)*
        #vis #fn_token #ident (suite: &mut rye::TestSuite<'_>) {
            #asyncness #fn_token #inner_fn_ident() #output #block
            let desc = rye::_internal::TestDesc {
                name: #test_name,
                module_path: module_path!(),
                sections: rye::_internal::hashmap! { #(#section_map_entries,)* },
                leaf_sections: &[ #(#leaf_section_ids),* ],
            };
            #register
        }
    }
}
