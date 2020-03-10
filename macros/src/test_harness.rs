use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, Path, Token, UseTree,
};

struct Input {
    test_runner: Path,
    test_cases: Vec<UseTree>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_inner(input)?;

        let mut test_runner = None;
        let mut test_cases = vec![];
        for attr in attrs {
            match attr.path.get_ident() {
                Some(id) if id == "test_runner" => {
                    test_runner.replace(attr.parse_args()?);
                }
                Some(id) if id == "test_cases" => {
                    let cases =
                        attr.parse_args_with(Punctuated::<UseTree, Token![,]>::parse_terminated)?;
                    test_cases.extend(cases);
                }
                _ => return Err(Error::new_spanned(&attr.path, "unsupported attribute")),
            }
        }

        Ok(Self {
            test_runner: test_runner.ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    "missing attribute: #![test_runner(path::to::runner)]",
                )
            })?,
            test_cases,
        })
    }
}

pub(crate) fn test_harness(input: TokenStream) -> TokenStream {
    let input: Input = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    let test_runner = input.test_runner;

    let test_cases = match crate::test_set::extract_test_cases(&input.test_cases) {
        Ok(paths) => paths,
        Err(err) => return err.to_compile_error(),
    };

    quote! {
        fn main() {
            #test_runner(&[ #( #test_cases ),* ]);
        }
    }
}
