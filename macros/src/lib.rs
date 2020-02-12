extern crate proc_macro;

mod test_case;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test_case(args: TokenStream, item: TokenStream) -> TokenStream {
    crate::test_case::test_case(args.into(), item.into()).into()
}
