use crate::section::{Section, SectionId};
use indexmap::IndexMap;
use std::mem;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAsync, ExprClosure, ExprForLoop, ExprLoop, ExprWhile, Item, ItemFn, Macro,
    Stmt, Token,
};

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

#[inline]
pub(crate) fn expand(item: &mut ItemFn) -> Vec<Section> {
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
}
