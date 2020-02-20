use proc_macro2::{Span, TokenStream};
use quote::quote;
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
                let errors = errors.to_compile_error();
                return quote! {
                    fn main() {
                        #errors
                    }
                };
            }
        };
    };
}

struct Input {
    test_cases: Punctuated<UseTree, Token![,]>,
    runner: Path,
    rye_path: Path,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut test_cases = None;
        let mut runner = None;
        let mut rye_path = None;

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            let _eq_token: Token![=] = input.parse()?;

            match &name {
                name if name == "test_cases" => {
                    if test_cases.is_some() {
                        return Err(Error::new_spanned(&name, "duplicated parameter"));
                    }
                    let content;
                    let _brace_token = syn::braced!(content in input);
                    test_cases.replace(Punctuated::parse_terminated(&content)?);
                }
                name if name == "runner" => {
                    if runner.is_some() {
                        return Err(Error::new_spanned(&name, "duplicated parameter"));
                    }
                    runner.replace(input.parse()?);
                }
                name if name == "rye_path" => {
                    if rye_path.is_some() {
                        return Err(Error::new_spanned(&name, "duplicated parameter"));
                    }
                    rye_path.replace(input.parse()?);
                }
                _ => {
                    return Err(Error::new_spanned(
                        &name,
                        format!("unknown parameter: '{}'", name),
                    ))
                }
            }

            let _: Token![;] = input.parse()?;
        }

        Ok(Self {
            test_cases: test_cases
                .ok_or_else(|| Error::new(Span::call_site(), "missing parameter: `test_cases'"))?,
            runner: runner.unwrap_or_else(|| syn::parse_quote!(::rye::_internal::default_runner)),
            rye_path: rye_path.unwrap_or_else(|| syn::parse_quote!(::rye)),
        })
    }
}

pub(crate) fn test_main(input: TokenStream) -> TokenStream {
    let input: Input = try_parse!(syn::parse2(input));
    let runner = &input.runner;
    let rye_path = &input.rye_path;
    let paths = try_parse!(extract_test_cases(&input));
    quote! {
        fn main() {
            #runner(&[#( & #paths as &dyn #rye_path::_internal::Registration ),*]);
        }
    }
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
