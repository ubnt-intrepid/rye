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
    runtime: Option<Path>,
    crate_path: Option<Path>,
}

impl Params {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut runtime: Option<Path> = None;
        let mut crate_path: Option<Path> = None;

        let mut errors: Option<Error> = None;
        attrs.retain(|attr| {
            if !attr.path.is_ident("rye") {
                return true;
            }

            let res = attr.parse_args_with(|input: ParseStream| -> Result<()> {
                match input.call(Ident::parse_any)? {
                    id if id == "runtime" => {
                        let _: Token![=] = input.parse()?;
                        let value = input.call(Path::parse_mod_style)?;
                        runtime.replace(value);
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
            runtime,
            crate_path,
        })
    }
}

pub(crate) fn test_main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item: ItemFn = try_quote!(syn::parse2(item));
    let params = try_quote!(Params::from_attrs(&mut item.attrs));

    let ident = &item.sig.ident;

    if item.sig.asyncness.is_none() {
        let err = Error::new_spanned(&item, "non-async function is not accepted");
        return err.to_compile_error();
    }

    let crate_path = params
        .crate_path
        .unwrap_or_else(|| syn::parse_quote!(::rye));

    let runtime = params
        .runtime
        .unwrap_or_else(|| syn::parse_quote!(__rye::default_runtime));
    // TODO: add type check.

    quote! {
        fn #ident() {
            #[allow(unused_imports)]
            use #crate_path::_test_main_reexports as __rye;

            #item

            __rye::install_globals();

            use __rye::Runtime as _;
            let mut rt = #runtime();
            let mut spawner = rt.spawner();
            __rye::exit(rt.block_on(async move {
                let mut data = __rye::SessionData::new();
                #ident(&mut data.session(&mut spawner)).await
            }));
        }
    }
}
