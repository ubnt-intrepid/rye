#![cfg_attr(docs, feature(doc_cfg))]

extern crate proc_macro;

mod test;
mod test_harness;

use proc_macro::TokenStream;

/// Generate a single test case.
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test::test(args.into(), item.into()).into()
}

/// Generate the main function for running the test cases.
#[cfg_attr(docs, doc(cfg(feature = "harness")))]
#[proc_macro]
pub fn test_harness(input: TokenStream) -> TokenStream {
    crate::test_harness::test_harness(input.into()).into()
}
