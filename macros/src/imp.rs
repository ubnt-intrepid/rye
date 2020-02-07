use crate::args::Args;
use proc_macro2::{Span, TokenStream};
use syn::{parse::Result, Ident, ItemFn};

const CURRENT_SECTION_IDENT_NAME: &str = "__rye_current_section__";
const INNER_FN_IDENT_NAME: &str = "__rye_inner_fn__";

pub(crate) fn test_case(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: Args = syn::parse2(args)?;

    let mut item: ItemFn = syn::parse2(item)?;

    let current_section_ident = Ident::new(CURRENT_SECTION_IDENT_NAME, Span::call_site());
    let inner_fn_ident = Ident::new(INNER_FN_IDENT_NAME, item.sig.ident.span());

    crate::expand::expand(&mut *item.block, &current_section_ident)?;

    Ok(crate::generate::generate(
        args,
        item,
        current_section_ident,
        inner_fn_ident,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::path::Path;

    fn read_file<P: AsRef<Path>>(path: P, expand_braces: bool) -> TokenStream {
        let mut content = std::fs::read_to_string(path).unwrap();
        if expand_braces {
            content = content
                .replace("{{section}}", CURRENT_SECTION_IDENT_NAME)
                .replace("{{inner}}", INNER_FN_IDENT_NAME);
        }
        content.parse().unwrap()
    }

    fn test_expanded(name: &str, args: impl Into<Option<TokenStream>>) {
        let args = args.into().unwrap_or_else(TokenStream::new);
        let item = read_file(format!("test/{}.in", name), false);
        let expected = read_file(format!("test/{}.out", name), true);
        let output = test_case(args, item).unwrap();
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn test_suite() {
        test_expanded("01-sync", None);
        test_expanded("02-nested", None);
        test_expanded("03-async", None);
        test_expanded(
            "04-async-with-args",
            quote!(block_on = "path::to::custom_block_on"),
        );
    }
}
