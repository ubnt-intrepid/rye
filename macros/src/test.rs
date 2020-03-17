use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt as _};
use std::mem;
use syn::{
    ext::IdentExt as _,
    parse::{Error, Parse, ParseStream, Parser as _, Result},
    spanned::Spanned as _,
    visit_mut::{self, VisitMut},
    Attribute, Block, Expr, ExprMacro, Ident, Item, ItemFn, Stmt, Token,
};

macro_rules! try_quote {
    ($e:expr) => {
        match $e {
            Ok(parsed) => parsed,
            Err(err) => return err.to_compile_error(),
        }
    };
}

pub(crate) fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = try_quote!(syn::parse2::<ItemFn>(item));

    match &item.sig.inputs {
        inputs if inputs.is_empty() => (),
        inputs => {
            return Error::new_spanned(inputs, "test functions cannot accept arguments")
                .to_compile_error()
        }
    }

    match &item.sig.generics {
        generics if generics.params.is_empty() => (),
        generics => {
            return Error::new_spanned(generics, "test functions cannot take generic parameters")
                .to_compile_error()
        }
    }

    if item.sig.asyncness.is_none() && !args.is_empty() {
        return Error::new_spanned(&args, "accepted only for async functions").to_compile_error();
    }
    let args = try_quote!(syn::parse2::<Args>(args));

    // extract rye-specific attributes.
    let params = try_quote!(Params::from_attrs(&mut item.attrs));

    // expand section!()
    let sections = expand_sections(&mut item);

    item.block
        .stmts
        .insert(0, Stmt::Item(params.reexport_internal_module()));

    let generated = Generated {
        item: &item,
        params: &params,
        args: &args,
        sections: &sections,
    };

    quote! {
        #item
        #generated
    }
}

struct Args {
    local: bool,
}

mod kw {
    syn::custom_keyword!(Send);
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self { local: false });
        }

        let span = input.span();
        let error = || Error::new(span, "only '?Send' or '!Send' is accepted");

        if input.peek(Token![?]) {
            let _: Token![?] = input.parse().unwrap();
        } else if input.peek(Token![!]) {
            let _: Token![!] = input.parse().unwrap();
        } else {
            return Err(error());
        }
        input.parse::<kw::Send>().map_err(|_| error())?;

        Ok(Args { local: true })
    }
}

struct Params {
    crate_path: syn::Path,
}

impl Params {
    fn reexport_internal_module(&self) -> syn::Item {
        let crate_path = &self.crate_path;
        syn::parse_quote! {
            #[allow(unused_imports)]
            use #crate_path::_internal as __rye;
        }
    }
}

impl Params {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut crate_path = None;

        let mut parse_attr = |input: ParseStream<'_>| -> Result<()> {
            match input.call(Ident::parse_any)? {
                id if id == "crate" => {
                    let _: Token![=] = input.parse()?;
                    let lit: syn::LitStr = input.parse()?;

                    if let Some(..) = crate_path {
                        return Err(Error::new_spanned(&id, "duplicated parameter"));
                    }

                    crate_path.replace(lit.parse()?);
                    Ok(())
                }
                id => Err(Error::new_spanned(id, "unknown parameter name")),
            }
        };

        let mut errors = Errors::default();
        attrs.retain(|attr| {
            if !attr.path.is_ident("rye") {
                return true;
            }
            errors.append_if_error(attr.parse_args_with(&mut parse_attr));
            false
        });
        errors.into_result()?;

        Ok(Self {
            crate_path: crate_path.unwrap_or_else(|| syn::parse_quote!(::rye)),
        })
    }
}

#[derive(Default)]
struct Errors(Option<Error>);

impl Errors {
    fn append_if_error(&mut self, res: Result<()>) {
        match (self.0.as_mut(), res) {
            (Some(errors), Err(error)) => errors.combine(error),
            (None, Err(error)) => self.0 = Some(error),
            (_, Ok(())) => (),
        }
    }

    fn into_result(self) -> Result<()> {
        match self.0 {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }
}

type SectionId = u64;

struct Section {
    id: SectionId,
    name: syn::Expr,
    ancestors: Vec<SectionId>,
    children: Vec<SectionId>,
}

fn expand_sections(item: &mut ItemFn) -> Vec<Section> {
    let mut expand = ExpandSections {
        sections: IndexMap::new(),
        next_section_id: 0,
        parent: None,
        in_loop: false,
        in_closure: false,
        in_async_block: false,
    };
    expand.visit_block_mut(&mut *item.block);
    expand.sections.into_iter().map(|(_k, v)| v).collect()
}

struct ExpandSections {
    sections: IndexMap<SectionId, Section>,
    next_section_id: SectionId,
    parent: Option<SectionId>,
    in_loop: bool,
    in_closure: bool,
    in_async_block: bool,
}

impl ExpandSections {
    fn expand_section_macro(&mut self, expr: &ExprMacro) -> Result<Stmt> {
        if self.in_loop {
            return Err(Error::new_spanned(
                expr,
                "section cannot be described in a loop",
            ));
        }
        if self.in_closure {
            return Err(Error::new_spanned(
                expr,
                "section cannot be described in a closure",
            ));
        }
        if self.in_async_block {
            return Err(Error::new_spanned(
                expr,
                "section cannot be described in an async block",
            ));
        }

        let attrs = &expr.attrs;

        let (name, mut block) =
            expr.mac
                .parse_body_with(|input: ParseStream<'_>| -> Result<_> {
                    let name: Expr = input.parse()?;
                    let _: Token![,] = input.parse()?;
                    let block: Box<Block> = input.parse()?;
                    Ok((name, block))
                })?;

        let section_id = self.next_section_id;
        let ancestors = if let Some(parent) = self.parent {
            let parent = &mut self.sections[&parent];
            parent.children.push(section_id);
            let mut ancestors = parent.ancestors.clone();
            ancestors.push(parent.id);
            ancestors
        } else {
            vec![]
        };
        self.sections.insert(
            section_id,
            Section {
                id: section_id,
                name,
                ancestors,
                children: vec![],
            },
        );
        self.next_section_id += 1;

        self.enter_section(section_id, |me| {
            me.visit_block_mut(&mut *block);
        });

        Stmt::parse.parse2(quote_spanned! { expr.mac.span() =>
            __rye::enter_section!(#section_id, #(#attrs)* #block);
        })
    }

    fn enter_section<F, R>(&mut self, section_id: SectionId, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = self.parent.replace(section_id);
        let res = f(self);
        self.parent = prev;
        res
    }
}

impl VisitMut for ExpandSections {
    fn visit_stmt_mut(&mut self, item: &mut Stmt) {
        match item {
            Stmt::Expr(Expr::Macro(expr_macro)) | Stmt::Semi(Expr::Macro(expr_macro), _)
                if expr_macro.mac.path.is_ident("section") =>
            {
                let stmt = match self.expand_section_macro(&*expr_macro) {
                    Ok(expanded) => expanded,
                    Err(err) => {
                        let err = err.to_compile_error();
                        syn::parse_quote!(#err)
                    }
                };
                mem::replace(item, stmt);
            }
            _ => visit_mut::visit_stmt_mut(self, item),
        }
    }

    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        let mut in_loop = self.in_loop;
        let mut in_closure = self.in_closure;
        let mut in_async_block = self.in_async_block;
        match node {
            Expr::ForLoop(..) | Expr::Loop(..) | Expr::While(..) => in_loop = true,
            Expr::Closure(..) => in_closure = true,
            Expr::Async(..) => in_async_block = true,
            _ => (),
        }

        let in_loop_prev = mem::replace(&mut self.in_loop, in_loop);
        let in_closure_prev = mem::replace(&mut self.in_closure, in_closure);
        let in_async_block_prev = mem::replace(&mut self.in_async_block, in_async_block);

        visit_mut::visit_expr_mut(self, node);

        self.in_loop = in_loop_prev;
        self.in_closure = in_closure_prev;
        self.in_async_block = in_async_block_prev;
    }

    fn visit_item_mut(&mut self, _node: &mut Item) {
        // ignore inner items.
    }
}

struct Generated<'a> {
    params: &'a Params,
    args: &'a Args,
    item: &'a ItemFn,
    sections: &'a [Section],
}

impl ToTokens for Generated<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        struct SectionMapEntry<'a> {
            section: &'a Section,
        }

        impl ToTokens for SectionMapEntry<'_> {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let id = &self.section.id;
                let name = &self.section.name;
                let ancestors = &self.section.ancestors;
                tokens.append_all(&[quote! {
                    #id => (#name, { #( #ancestors ),* });
                }]);
            }
        }

        let section_map_entries = self
            .sections
            .iter()
            .map(|section| SectionMapEntry { section });
        let leaf_section_ids = self.sections.iter().filter_map(|section| {
            if section.children.is_empty() {
                Some(section.id)
            } else {
                None
            }
        });

        let ident = &self.item.sig.ident;
        let rye_reexport = &self.params.reexport_internal_module();
        let location = quote_spanned!(self.item.sig.span() => __rye::location!());

        let test_fn_id = match self.item.sig.asyncness {
            Some(..) if self.args.local => Ident::new("async_local_test_fn", Span::call_site()),
            Some(..) => Ident::new("async_test_fn", Span::call_site()),
            None => Ident::new("blocking_test_fn", Span::call_site()),
        };

        tokens.append_all(vec![quote! {
            pub(crate) mod #ident {
                use super::*;
                #rye_reexport

                #[allow(non_camel_case_types)]
                pub(crate) struct __tests(());

                impl __tests {
                    pub(crate) const fn new() -> Self {
                        Self(())
                    }
                }

                impl __rye::TestSet for __tests {
                    fn register(&self, __registry: &mut dyn __rye::Registry) -> __rye::Result<(), __rye::RegistryError> {
                        __registry.add_test(
                            __rye::TestDesc {
                                module_path: __rye::module_path!(),
                                location: #location,
                                sections: __rye::declare_section! { #( #section_map_entries )* },
                                leaf_sections: &[ #( #leaf_section_ids ),* ],
                            },
                            __rye::#test_fn_id!(#ident)
                        )?;
                        __rye::Result::Ok(())
                    }
                }

                __rye::cfg_frameworks! {
                    #[test_case]
                    static __TESTS: &dyn __rye::TestSet = &__tests::new();
                }
            }
        }]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse_items(input: ParseStream) -> Result<Vec<Item>> {
        let mut items = vec![];
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(items)
    }

    fn read_file<P: AsRef<Path>>(path: P) -> TokenStream {
        let content = std::fs::read_to_string(path).unwrap();
        let items = parse_items.parse_str(&content).unwrap();
        quote!(#(#items)*)
    }

    fn test_expanded(name: &str) {
        let args = TokenStream::new();
        let item = read_file(format!("tests/test/{}.in.rs", name));
        let expected = read_file(format!("tests/test/{}.out.rs", name));
        let output = test(args, item);
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn test_sync() {
        test_expanded("01-sync");
    }

    #[test]
    fn test_sync_nested() {
        test_expanded("02-sync-nested");
    }

    #[test]
    fn test_async() {
        test_expanded("03-async");
    }

    #[test]
    fn test_async_nested() {
        test_expanded("04-async-nested");
    }

    #[test]
    fn multi_sections_in_scope() {
        test_expanded("05-multi-sections-in-scope");
    }

    #[test]
    fn ignore_inner_items() {
        test_expanded("06-ignore-inner-items");
    }

    #[test]
    fn no_sections() {
        test_expanded("07-no-sections");
    }

    #[test]
    fn attributes() {
        test_expanded("08-attributes");
    }
}
