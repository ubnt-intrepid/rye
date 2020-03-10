use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt as _};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, Token, UseTree,
};

macro_rules! try_parse {
    ($e:expr) => {
        match $e {
            Ok(ret) => ret,
            Err(errors) => {
                return Generated {
                    body: errors.to_compile_error(),
                }
                .into_token_stream();
            }
        };
    };
}

struct Input {
    test_cases: Vec<UseTree>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_inner(input)?;
        let mut test_cases = vec![];
        for attr in attrs {
            match attr.path.get_ident() {
                Some(id) if id == "test_cases" => {
                    let cases =
                        attr.parse_args_with(Punctuated::<UseTree, Token![,]>::parse_terminated)?;
                    test_cases.extend(cases);
                }
                _ => return Err(Error::new_spanned(attr, "unknown attribute")),
            }
        }
        Ok(Self { test_cases })
    }
}

pub(crate) fn test_module(input: TokenStream) -> TokenStream {
    let input: Input = try_parse!(syn::parse2(input));
    let paths = try_parse!(crate::common::extract_test_cases(input.test_cases.iter()));
    Generated {
        body: quote! {
            for tests in &[#( & #paths as &dyn rye::_internal::TestSet ),*] {
                tests.register(__registry)?;
            }
            Ok(())
        },
    }
    .into_token_stream()
}

struct Generated<T: ToTokens> {
    body: T,
}

impl<T> ToTokens for Generated<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let body = &self.body;
        tokens.append_all(&[
            quote! {
                struct __tests(());
                impl ::rye::_internal::TestSet for __tests {
                    fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> Result<(), ::rye::_internal::RegistryError> {
                        #body
                    }
                }
                pub(crate) const __TESTS: &dyn ::rye::_internal::TestSet = &__tests(());
            }
        ]);
    }
}
