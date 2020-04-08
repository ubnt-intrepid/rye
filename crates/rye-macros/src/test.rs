use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt as _};
use std::mem;
use syn::{
    ext::IdentExt as _,
    parse::{Error, Parse, ParseStream, Result},
    spanned::Spanned as _,
    visit_mut::{self, VisitMut},
    Attribute, Block, Expr, ExprMacro, Ident, Item, ItemFn, ItemMacro, Macro, Path, Stmt, Token,
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
    let sections = expand_builtins(&mut item);

    // append bounds to where clause.
    if let syn::ReturnType::Type(_, ref ty) = item.sig.output {
        let where_clause = item.sig.generics.make_where_clause();
        where_clause
            .predicates
            .push(syn::parse_quote!(#ty: __rye::Termination));
    }

    Generated {
        item: &item,
        params: &params,
        args: &args,
        sections: &sections,
    }
    .to_token_stream()
}

#[derive(Copy, Clone)]
enum Sendness {
    Send,
    NoSend,
}

struct Args {
    sendness: Sendness,
}

mod kw {
    syn::custom_keyword!(Send);
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                sendness: Sendness::Send,
            });
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

        Ok(Args {
            sendness: Sendness::NoSend,
        })
    }
}

struct Params {
    crate_path: Path,
}

impl Params {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut crate_path = None;

        let mut parse_attr = |input: ParseStream<'_>| -> Result<()> {
            match input.call(Ident::parse_any)? {
                id if id == "crate" => {
                    let _: Token![=] = input.parse()?;
                    let path = input.call(Path::parse_mod_style)?;
                    crate_path.replace(path);
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
    ancestors: Vec<SectionId>,
    children: Vec<SectionId>,
}

fn expand_builtins(item: &mut ItemFn) -> Vec<Section> {
    let mut expand = ExpandBuiltins {
        sections: IndexMap::new(),
        next_section_id: 0,
        parent: None,
        forbidden_sections: false,
        block_state: BlockState::Setup,
    };
    expand.visit_block_mut(&mut *item.block);
    expand.sections.into_iter().map(|(_k, v)| v).collect()
}

struct ExpandBuiltins {
    sections: IndexMap<SectionId, Section>,
    next_section_id: SectionId,
    parent: Option<SectionId>,
    forbidden_sections: bool,
    block_state: BlockState,
}

enum BlockState {
    Setup,
    Sections,
    Teardown,
}

impl ExpandBuiltins {
    fn try_expand_section(&mut self, attrs: &[Attribute], mac: &Macro) -> Result<Stmt> {
        if self.forbidden_sections {
            return Err(Error::new_spanned(
                mac,
                "section cannot be described at here",
            ));
        }

        match self.block_state {
            BlockState::Setup | BlockState::Sections => {
                self.block_state = BlockState::Sections;
            }
            BlockState::Teardown => {
                return Err(Error::new_spanned(
                    mac,
                    "section cannot be described after teardown",
                ));
            }
        }

        let (ctx, name, mut block) =
            mac.parse_body_with(|input: ParseStream<'_>| -> Result<_> {
                let ctx: Ident = input.parse()?;
                let _: Token![,] = input.parse()?;
                let name: Expr = input.parse()?;
                let _: Token![,] = input.parse()?;
                let block: Box<Block> = input.parse()?;
                Ok((ctx, name, block))
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
                ancestors,
                children: vec![],
            },
        );
        self.next_section_id += 1;

        self.enter_section(section_id, |me| {
            me.visit_block_mut(&mut *block);
        });

        Ok(Stmt::Item(Item::Verbatim(quote_spanned! { mac.span() =>
            __rye::section!(#ctx, #section_id, #name, #(#attrs)* #block);
        })))
    }

    fn expand_section(&mut self, attrs: &[Attribute], mac: &Macro) -> Stmt {
        self.try_expand_section(attrs, mac)
            .unwrap_or_else(|err| Stmt::Item(Item::Verbatim(err.to_compile_error())))
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

    fn forbid_sections<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = mem::replace(&mut self.forbidden_sections, true);
        let res = f(self);
        self.forbidden_sections = prev;
        res
    }

    fn with_block_state<F, R>(&mut self, state: BlockState, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = mem::replace(&mut self.block_state, state);
        let res = f(self);
        self.block_state = prev;
        res
    }
}

impl VisitMut for ExpandBuiltins {
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.with_block_state(BlockState::Setup, |me| {
            visit_mut::visit_block_mut(me, block);
        });
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            #[rustfmt::skip]
            | Stmt::Expr(Expr::Macro(ExprMacro { attrs, mac, .. }))
            | Stmt::Semi(Expr::Macro(ExprMacro { attrs, mac, .. }), _)
            | Stmt::Item(Item::Macro(ItemMacro { attrs, mac, .. }))
                if mac.path.is_ident("section") =>
            {
                *stmt = self.expand_section(attrs, mac);
            }

            Stmt::Item(..) => { /* ignore inner items */ }

            stmt => {
                match self.block_state {
                    BlockState::Setup | BlockState::Teardown => (),
                    BlockState::Sections => self.block_state = BlockState::Teardown,
                }
                self.forbid_sections(|me| {
                    visit_mut::visit_stmt_mut(me, stmt);
                });
            }
        }
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
        let mut plans: Vec<_> = self
            .sections
            .iter()
            .filter_map(|section| {
                if section.children.is_empty() {
                    let target = section.id;
                    let ancestors = &section.ancestors;
                    Some(quote! {
                        __rye::TestPlan {
                            target: Some(#target),
                            ancestors: &[ #(#ancestors),* ],
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        if plans.is_empty() {
            plans.push(quote! {
                __rye::TestPlan {
                    target: None,
                    ancestors: &[],
                }
            });
        }

        let crate_path = &self.params.crate_path;
        let item = &*self.item;
        let ident = &self.item.sig.ident;
        let location = quote_spanned!(self.item.sig.span() => __rye::location!());

        let test_fn_id = match (self.item.sig.asyncness, self.args.sendness) {
            (Some(..), Sendness::Send) => Ident::new("async", Span::call_site()),
            (Some(..), Sendness::NoSend) => Ident::new("async_local", Span::call_site()),
            (None, ..) => Ident::new("blocking", Span::call_site()),
        };

        let test_case_id = quote::format_ident!("__TEST_CASE_{}", ident);

        tokens.append_all(Some(quote! {
            #[allow(non_upper_case_globals)]
            const #ident: & #crate_path::_test_reexports::TestCase = {
                #[allow(unused_imports)]
                use #crate_path::_test_reexports as __rye;

                #item

                &__rye::TestCase {
                    desc: __rye::TestDesc {
                        name: __rye::test_name!(#ident),
                        location: #location,
                    },
                    testfn: __rye::test_fn!(@#test_fn_id #ident),
                    plans: &[ #(#plans,)* ],
                }
            };
        }));

        tokens.append_all(Some(quote! {
            #crate_path::__test_case! {
                #[allow(non_upper_case_globals)]
                static #test_case_id: & #crate_path::_test_reexports::TestCase = #ident;
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use syn::parse::Parser as _;

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

    #[test]
    fn return_result() {
        test_expanded("09-return-result");
    }
}
