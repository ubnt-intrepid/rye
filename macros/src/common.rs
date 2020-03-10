use proc_macro2::Span;
use syn::{parse, punctuated::Punctuated, Ident, Path, Token, UseGroup, UseName, UsePath, UseTree};

pub(crate) fn extract_test_cases(
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
