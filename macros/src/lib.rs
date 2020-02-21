extern crate proc_macro;

mod test;
mod test_group;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test::test(args.into(), item.into()).into()
}

#[proc_macro]
pub fn test_group(input: TokenStream) -> TokenStream {
    crate::test_group::test_group(input.into()).into()
}
