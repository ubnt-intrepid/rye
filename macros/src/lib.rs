extern crate proc_macro;

mod common;
mod test;
mod test_harness;
mod test_module;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test::test(args.into(), item.into()).into()
}

#[proc_macro]
pub fn test_harness(input: TokenStream) -> TokenStream {
    crate::test_harness::test_harness(input.into()).into()
}

#[proc_macro]
pub fn test_module(input: TokenStream) -> TokenStream {
    crate::test_module::test_module(input.into()).into()
}
