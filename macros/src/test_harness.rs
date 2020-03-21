use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Error, Parse, ParseStream, Parser, Result},
    Attribute, Ident, Path, Token,
};

struct Input {
    test_runner: Path,
    reexport_test_harness_main: Option<Ident>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_inner(input)?;

        let mut test_runner = None;
        let mut reexport_test_harness_main = None;
        for attr in attrs {
            match attr.path.get_ident() {
                Some(id) if id == "test_runner" => {
                    test_runner.replace(attr.parse_args()?);
                }
                Some(id) if id == "reexport_test_harness_main" => {
                    fn parse(input: ParseStream<'_>) -> Result<Ident> {
                        let _eq_token: Token![=] = input.parse()?;
                        let value: syn::LitStr = input.parse()?;
                        value.parse()
                    }
                    let main_id = parse.parse2(attr.tokens)?;
                    reexport_test_harness_main.replace(main_id);
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
            reexport_test_harness_main,
        })
    }
}

pub(crate) fn test_harness(input: TokenStream) -> TokenStream {
    let input: Input = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    let test_runner = input.test_runner;
    let main_id = input
        .reexport_test_harness_main
        .unwrap_or_else(|| format_ident!("main"));

    quote! {
        #[cfg(any(test, trybuild))]
        fn #main_id () {
            #test_runner(&::rye::_internal::TEST_CASES[..]);
        }
    }
}
