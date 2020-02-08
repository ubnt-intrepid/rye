use proc_macro2::TokenStream;
use syn::{parse::Result, ItemFn};

pub(crate) fn test_case(_args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut item: ItemFn = syn::parse2(item)?;
    crate::expand::expand(&mut item);
    Ok(crate::generate::generate(item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn read_file<P: AsRef<Path>>(path: P) -> TokenStream {
        let content = std::fs::read_to_string(path).unwrap();
        let item: syn::ItemFn = syn::parse_str(&content).unwrap();
        quote::quote!(#item)
    }

    fn test_expanded(name: &str) {
        let args = TokenStream::new();
        let item = read_file(format!("tests/expand/{}.in.rs", name));
        let expected = read_file(format!("tests/expand/{}.out.rs", name));
        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn test_sync() {
        test_expanded("01-sync");
    }

    #[test]
    fn test_sync_nested() {
        test_expanded("02-sync-nested");
    }

    #[test]
    fn test_async() {
        test_expanded("03-async");
    }

    #[test]
    fn test_async_nested() {
        test_expanded("04-async-nested");
    }

    #[test]
    fn multi_sections_in_scope() {
        test_expanded("05-multi-sections-in-scope");
    }
}
