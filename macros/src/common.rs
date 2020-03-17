use proc_macro2::Span;
use syn::{
    parse, punctuated::Punctuated, Attribute, Expr, Ident, Token, UseGroup, UseName, UsePath,
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

    pub(crate) fn extract_test_cases(&self) -> parse::Result<Vec<Expr>> {
        extract_test_cases(&self.test_cases)
    }
}

fn extract_test_cases(
    trees: impl IntoIterator<Item = impl std::ops::Deref<Target = UseTree>>,
) -> parse::Result<Vec<Expr>> {
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
    test_cases: &mut Vec<Expr>,
    ancestors: &[&Ident],
    errors: &mut Vec<parse::Error>,
) {
    match tree {
        UseTree::Name(UseName { ident }) => {
            let path: Punctuated<&Ident, Token![::]> =
                ancestors.iter().copied().chain(Some(ident)).collect();
            test_cases.push(syn::parse_quote!(#path::__new()));
        }
        UseTree::Glob(..) => {
            #[allow(nonstandard_style)]
            let __tests = syn::Ident::new("__tests", Span::call_site());
            let path: Punctuated<&Ident, Token![::]> =
                ancestors.iter().copied().chain(Some(&__tests)).collect();
            test_cases.push(syn::parse_quote!(#path::__new()));
        }
        UseTree::Path(UsePath { ident, tree, .. }) => {
            let ancestors: Vec<_> = ancestors.iter().copied().chain(Some(ident)).collect();
            expand_use_tree(&*tree, test_cases, &ancestors[..], errors);
        }
        UseTree::Group(UseGroup { items, .. }) => {
            for tree in items {
                expand_use_tree(&*tree, test_cases, ancestors, errors);
            }
        }
        UseTree::Rename(rename) => errors.push(parse::Error::new_spanned(
            rename,
            "test cases cannot be renamed",
        )),
    }
}
