use crate::args::Args;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub(crate) fn generate(args: Args, item: ItemFn) -> TokenStream {
    let vis = &item.vis;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let test_attr: syn::Attribute = syn::parse_quote!(#[test]);
    let attrs = Some(&test_attr).into_iter().chain(&item.attrs);

    let body = match item.sig.asyncness {
        Some(ref async_token) => {
            let block_on: syn::Path = match args
                .values
                .iter()
                .find(|value| value.path.is_ident("block_on"))
            {
                Some(value) => match &value.lit {
                    syn::Lit::Str(lit_str) => match lit_str.parse() {
                        Ok(path) => path,
                        Err(err) => return err.to_compile_error().into(),
                    },
                    lit => {
                        use syn::spanned::Spanned;
                        let span = lit.span();
                        return quote::quote_spanned!(span => const _: () = compile_error!("should be a string literal");)
                            .into();
                    }
                },
                None => syn::parse_quote!(block_on),
            };

            quote! {
                #async_token #fn_token __inner__() #output #block
                #block_on(async {
                    let mut test_case = rye::TestCase::new();
                    while !test_case.completed() {
                        test_case.run_async(__inner__()).await;
                    }
                });
            }
        }
        None => quote! {
            #fn_token __inner__() #output #block
            let mut test_case = rye::TestCase::new();
            while !test_case.completed() {
                test_case.run(__inner__);
            }
        },
    };

    quote! {
        #(#attrs)*
        #vis #fn_token #ident () {
            #body
        }
    }
}
