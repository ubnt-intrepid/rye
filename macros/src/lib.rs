extern crate proc_macro;

mod test;
mod test_runner;
mod test_set;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test::test(args.into(), item.into()).into()
}

#[proc_macro]
pub fn test_set(input: TokenStream) -> TokenStream {
    crate::test_set::test_set(input.into()).into()
}

#[proc_macro]
pub fn test_runner(input: TokenStream) -> TokenStream {
    crate::test_runner::test_runner(input.into()).into()
}
