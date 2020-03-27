extern crate proc_macro;

mod test;
mod test_main;

use proc_macro::TokenStream;

/// Generate a single test case.
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test::test(args.into(), item.into()).into()
}

/// Define a test main function.
#[proc_macro_attribute]
pub fn test_main(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test_main::test_main(args.into(), item.into()).into()
}
