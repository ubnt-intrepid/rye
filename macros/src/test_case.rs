use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt as _};
use std::mem;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAsync, ExprClosure, ExprForLoop, ExprLoop, ExprWhile, Ident, Item, ItemFn,
    Macro, Stmt, Token,
};

pub(crate) fn test_case(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item: ItemFn = match syn::parse2(item) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let sections: Vec<_> = {
        let mut expand = ExpandBlock {
            sections: IndexMap::new(),
            next_section_id: 0,
            parent: None,
            in_loop: false,
            in_closure: false,
            in_async_block: false,
        };
        expand.visit_block_mut(&mut *item.block);
        expand.sections.into_iter().map(|(_k, v)| v).collect()
    };
    let sections = &sections;

    let vis = &item.vis;
    let asyncness = &item.sig.asyncness;
    let fn_token = &item.sig.fn_token;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let block = &*item.block;

    let inner_fn_ident = Ident::new("__inner__", ident.span());

    let register = if asyncness.is_some() {
        quote! {
            suite.register_async(desc, #inner_fn_ident);
        }
    } else {
        quote! {
            suite.register(desc, #inner_fn_ident);
        }
    };

    let test_name = ident.to_string();

    let section_map_entries = sections.iter().map(|section| SectionMapEntry { section });

    let leaf_section_ids = sections.iter().filter_map(|section| {
        if section.children.is_empty() {
            Some(section.id)
        } else {
            None
        }
    });

    let mut ignored = false;
    let mut attrs = vec![];
    for attr in item.attrs.drain(..) {
        if attr.path.is_ident("ignored") {
            ignored = true;
            continue;
        }
        attrs.push(attr);
    }

    quote! {
        #vis #fn_token #ident (suite: &mut rye::_internal::TestSuite<'_>) {
            #(#attrs)*
            #asyncness #fn_token #inner_fn_ident() #output #block

            let desc = rye::_internal::TestDesc {
                name: #test_name,
                module_path: module_path!(),
                ignored: #ignored,
                sections: rye::_internal::hashmap! { #(#section_map_entries,)* },
                leaf_sections: &[ #(#leaf_section_ids),* ],
            };
            #register
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

struct SectionBody {
    name: Expr,
    _comma: Token![,],
    block: Box<Block>,
}

impl Parse for SectionBody {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _comma: input.parse()?,
            block: input.parse()?,
        })
    }
}

struct ExpandBlock {
    sections: IndexMap<SectionId, Section>,
    next_section_id: SectionId,
    parent: Option<SectionId>,
    in_loop: bool,
    in_closure: bool,
    in_async_block: bool,
}

impl ExpandBlock {
    fn expand_section_macro(&mut self, mac: &Macro) -> Result<(Stmt, SectionId)> {
        if self.in_loop {
            return Err(Error::new_spanned(
                mac,
                "section cannot be described in a loop",
            ));
        }
        if self.in_closure {
            return Err(Error::new_spanned(
                mac,
                "section cannot be described in a closure",
            ));
        }
        if self.in_async_block {
            return Err(Error::new_spanned(
                mac,
                "section cannot be described in an async block",
            ));
        }

        let body: SectionBody = mac.parse_body()?;

        let name = &body.name;
        let block = &body.block;

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
                name: name.clone(),
                ancestors,
                children: vec![],
            },
        );
        self.next_section_id += 1;

        Ok((
            syn::parse_quote! {
                if rye::_internal::is_target(#section_id) #block
            },
            section_id,
        ))
    }

    fn mark_in_loop<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = mem::replace(&mut self.in_loop, true);
        let res = f(self);
        self.in_loop = prev;
        res
    }

    fn mark_in_closure<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = mem::replace(&mut self.in_closure, true);
        let res = f(self);
        self.in_closure = prev;
        res
    }

    fn mark_in_async_block<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = mem::replace(&mut self.in_async_block, true);
        let res = f(self);
        self.in_async_block = prev;
        res
    }
}

impl VisitMut for ExpandBlock {
    fn visit_stmt_mut(&mut self, item: &mut Stmt) {
        let section_id = match item {
            Stmt::Expr(Expr::Macro(expr_macro)) | Stmt::Semi(Expr::Macro(expr_macro), _) => {
                if expr_macro.mac.path.is_ident("section") {
                    let (stmt, section_id) = match self.expand_section_macro(&expr_macro.mac) {
                        Ok((expanded, section_id)) => (expanded, Some(section_id)),
                        Err(err) => {
                            let err = err.to_compile_error();
                            (syn::parse_quote!(#err), None)
                        }
                    };
                    mem::replace(item, stmt);
                    section_id
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(section_id) = section_id {
            let prev = self.parent.replace(section_id);
            visit_mut::visit_stmt_mut(self, item);
            self.parent = prev;
        } else {
            visit_mut::visit_stmt_mut(self, item);
        }
    }

    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        self.mark_in_loop(|me| visit_mut::visit_expr_for_loop_mut(me, node));
    }

    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        self.mark_in_loop(|me| visit_mut::visit_expr_loop_mut(me, node));
    }

    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        self.mark_in_loop(|me| visit_mut::visit_expr_while_mut(me, node));
    }

    fn visit_expr_closure_mut(&mut self, node: &mut ExprClosure) {
        self.mark_in_closure(|me| visit_mut::visit_expr_closure_mut(me, node));
    }

    fn visit_expr_async_mut(&mut self, node: &mut ExprAsync) {
        self.mark_in_async_block(|me| visit_mut::visit_expr_async_mut(me, node));
    }

    fn visit_item_mut(&mut self, _node: &mut Item) {
        // ignore inner items.
    }
}

struct SectionMapEntry<'a> {
    section: &'a Section,
}

impl ToTokens for SectionMapEntry<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.section.id;
        let name = &self.section.name;
        let ancestors = &self.section.ancestors;
        tokens.append_all(&[quote! {
            #id => rye::_internal::Section::new(
                #name,
                rye::_internal::hashset!(#(#ancestors),*)
            )
        }]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn read_file<P: AsRef<Path>>(path: P) -> TokenStream {
        let content = std::fs::read_to_string(path).unwrap();
        let item: syn::ItemFn = syn::parse_str(&content).unwrap();
        quote::quote!(#item)
    }

    fn test_expanded(name: &str) {
        let args = TokenStream::new();
        let item = read_file(format!("tests/test_case/{}.in.rs", name));
        let expected = read_file(format!("tests/test_case/{}.out.rs", name));
        let output = test_case(args, item);
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
