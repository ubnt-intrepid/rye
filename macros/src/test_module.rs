use crate::common::TestCases;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    Attribute,
};

struct Input {
    test_cases: TestCases,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_inner(input)?;

        let mut test_cases = TestCases::default();
        for attr in attrs {
            match attr.path.get_ident() {
                Some(id) if id == "test_cases" => {
                    test_cases.append_cases(&attr)?;
                }
                _ => return Err(Error::new_spanned(attr, "unknown attribute")),
            }
        }

        Ok(Self { test_cases })
    }
}

pub(crate) fn test_module(input: TokenStream) -> TokenStream {
    let input: Input = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };
    let test_cases = match input.test_cases.extract_test_cases() {
        Ok(cases) => cases,
        Err(err) => return err.to_compile_error(),
    };
    quote! {
        #[cfg(any(test, trybuild))]
        pub(crate) struct __tests(());

        #[cfg(any(test, trybuild))]
        impl __tests {
            pub(crate) const fn __new() -> Self {
                Self(())
            }
        }

        #[cfg(any(test, trybuild))]
        impl ::rye::_internal::TestSet for __tests {
            fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> Result<(), ::rye::_internal::RegistryError> {
                #(
                    #test_cases.register(__registry)?;
                )*
                Ok(())
            }
        }
    }
}
