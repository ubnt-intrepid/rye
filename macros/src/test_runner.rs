use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    Path, Token,
};

struct Input {
    runner: Path,
    target: Option<Path>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let runner = input.parse()?;

        let mut target = None;
        if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;
            target.replace(input.parse()?);
        }

        Ok(Self { runner, target })
    }
}

pub(crate) fn test_runner(input: TokenStream) -> TokenStream {
    let input: Input = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    let runner = input.runner;

    let mut registration = input.target.unwrap_or_else(|| syn::parse_quote!(self));
    registration
        .segments
        .push(syn::parse_quote!(__REGISTRATION));

    quote! {
        fn main() {
            #runner(&[ #registration ]);
        }
    }
}
