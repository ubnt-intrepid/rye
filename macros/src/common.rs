use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt as _};
use syn::{
    parse, punctuated::Punctuated, Attribute, Ident, Path, Token, UseGroup, UseName, UsePath,
    UseTree,
};

#[derive(Default)]
pub(crate) struct TestCases {
    test_cases: Vec<UseTree>,
}

impl TestCases {
    pub(crate) fn append_cases(&mut self, attr: &Attribute) -> parse::Result<()> {
        let cases = attr.parse_args_with(Punctuated::<UseTree, Token![,]>::parse_terminated)?;
        self.test_cases.extend(cases);
        Ok(())
    }
}

impl ToTokens for TestCases {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

        let body = extract_test_cases(&self.test_cases)
            .map(|paths| {
                quote! {
                    #( (#paths as &dyn rye::_internal::TestSet).register(__registry)?; )*
                    Ok(())
                }
            })
            .unwrap_or_else(|err| err.to_compile_error());

        tokens.append_all(&[Generated { body }]);
    }
}

fn extract_test_cases(
    trees: impl IntoIterator<Item = impl std::ops::Deref<Target = UseTree>>,
) -> parse::Result<Vec<Path>> {
    let mut paths = vec![];
    let mut errors = vec![];

    for tree in trees {
        expand_use_tree(&*tree, &mut paths, &[], &mut errors);
    }

    let errors = errors
        .into_iter()
        .fold(None::<parse::Error>, |mut errors, error| {
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
    errors: &mut Vec<parse::Error>,
) {
    match tree {
        UseTree::Name(UseName { ident }) => {
            #[allow(nonstandard_style)]
            let __TESTS = syn::Ident::new("__TESTS", Span::call_site());
            let path: Punctuated<&Ident, Token![::]> = ancestors
                .iter()
                .copied()
                .chain(Some(ident))
                .chain(Some(&__TESTS))
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
        UseTree::Rename(rename) => errors.push(parse::Error::new_spanned(
            rename,
            "rename pattern is forbidden",
        )),
        UseTree::Glob(glob) => {
            errors.push(parse::Error::new_spanned(glob, "glob pattern is forbidden"))
        }
    }
}
