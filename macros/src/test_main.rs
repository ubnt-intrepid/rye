use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    ext::IdentExt as _,
    parse::{Error, ParseStream, Result},
    Attribute, Ident, ItemFn, Path, Token,
};

macro_rules! try_quote {
    ($e:expr) => {
        match $e {
            Ok(parsed) => parsed,
            Err(err) => return err.to_compile_error(),
        }
    };
}

#[derive(Default)]
struct Params {
    block_on: Option<Path>,
    crate_path: Option<Path>,
}

impl Params {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut block_on: Option<Path> = None;
        let mut crate_path: Option<Path> = None;

        let mut errors: Option<Error> = None;
        attrs.retain(|attr| {
            if !attr.path.is_ident("rye") {
                return true;
            }

            let res = attr.parse_args_with(|input: ParseStream| -> Result<()> {
                match input.call(Ident::parse_any)? {
                    id if id == "block_on" => {
                        let _: Token![=] = input.parse()?;
                        let value = input.call(Path::parse_mod_style)?;
                        block_on.replace(value);
                        Ok(())
                    }
                    id if id == "crate" => {
                        let _: Token![=] = input.parse()?;
                        let value = input.call(Path::parse_mod_style)?;
                        crate_path.replace(value);
                        Ok(())
                    }
                    id => Err(Error::new_spanned(&id, "expected `block_on`")),
                }
            });

            if let Err(err) = res {
                match errors {
                    Some(ref mut errors) => errors.combine(err),
                    None => errors = Some(err),
                }
            }

            false
        });

        if let Some(errors) = errors {
            return Err(errors);
        }

        Ok(Self {
            block_on,
            crate_path,
        })
    }
}

pub(crate) fn test_main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item: ItemFn = try_quote!(syn::parse2(item));
    let params = try_quote!(Params::from_attrs(&mut item.attrs));

    let ident = &item.sig.ident;

    let crate_path = params
        .crate_path
        .unwrap_or_else(|| syn::parse_quote!(::rye));

    let block_on = params
        .block_on
        .unwrap_or_else(|| syn::parse_quote!(__rye::default_block_on));

    quote! {
        fn #ident() {
            #[allow(unused_imports)]
            use #crate_path::_test_main_reexports as __rye;

            #item

            __rye::exit(#block_on(|mut exec| {
                async move {
                    let mut runner = __rye::TestRunner::new(&mut *exec);
                    #ident(&mut runner).await
                }
            }));
        }
    }
}
