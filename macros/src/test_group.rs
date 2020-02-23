use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt as _};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Path, Token, UseGroup, UseName, UsePath, UseTree,
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
    test_cases: Punctuated<UseTree, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            test_cases: Punctuated::parse_terminated(input)?,
        })
    }
}

pub(crate) fn test_group(input: TokenStream) -> TokenStream {
    let input: Input = try_parse!(syn::parse2(input));
    let paths = try_parse!(extract_test_cases(&input));
    Generated {
        body: quote! {
            let registrations = &[#( & #paths as &dyn rye::_internal::Registration ),*];
            for registration in registrations {
                registration.register(__registry)?;
            }
            Ok(())
        },
    }
    .into_token_stream()
}

fn extract_test_cases(input: &Input) -> Result<Vec<Path>> {
    let mut paths = vec![];
    let mut errors = vec![];

    for tree in &input.test_cases {
        expand_use_tree(tree, &mut paths, &[], &mut errors);
    }

    let errors = errors.into_iter().fold(None::<Error>, |mut errors, error| {
        if let Some(ref mut errors) = errors {
            errors.combine(error);
        } else {
            errors.replace(error);
        }
        errors
    });

    match errors {
        Some(errors) => Err(errors),
        None => Ok(paths),
    }
}

fn expand_use_tree(
    tree: &UseTree,
    paths: &mut Vec<Path>,
    ancestors: &[&Ident],
    errors: &mut Vec<Error>,
) {
    match tree {
        UseTree::Name(UseName { ident }) => {
            #[allow(nonstandard_style)]
            let __REGISTRATION = syn::Ident::new("__REGISTRATION", Span::call_site());
            let path: Punctuated<&Ident, Token![::]> = ancestors
                .iter()
                .copied()
                .chain(Some(ident))
                .chain(Some(&__REGISTRATION))
                .collect();
            paths.push(syn::parse_quote!(#path));
        }
        UseTree::Path(UsePath { ident, tree, .. }) => {
            let ancestors: Vec<_> = ancestors.iter().copied().chain(Some(ident)).collect();
            expand_use_tree(&*tree, paths, &ancestors[..], errors);
        }
        UseTree::Group(UseGroup { items, .. }) => {
            for tree in items {
                expand_use_tree(&*tree, paths, ancestors, errors);
            }
        }
        UseTree::Rename(rename) => {
            errors.push(Error::new_spanned(rename, "rename pattern is forbidden"))
        }
        UseTree::Glob(glob) => errors.push(Error::new_spanned(glob, "glob pattern is forbidden")),
    }
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
                struct __registration(());
                impl ::rye::_internal::Registration for __registration {
                    fn register(&self, __registry: &mut dyn ::rye::_internal::Registry) -> Result<(), ::rye::_internal::RegistryError> {
                        #body
                    }
                }
                pub(crate) const __REGISTRATION: &dyn ::rye::_internal::Registration = &__registration(());
            }
        ]);
    }
}