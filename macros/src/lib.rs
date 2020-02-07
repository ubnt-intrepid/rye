extern crate proc_macro;

mod expand;
mod generate;
mod imp;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test_case(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::imp::test_case(args.into(), item.into())
        .map(Into::into)
        .unwrap_or_else(|err| err.to_compile_error().into())
}
