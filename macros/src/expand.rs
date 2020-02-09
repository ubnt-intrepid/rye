use crate::section::{Section, SectionId};
use indexmap::IndexMap;
use std::mem;
use syn::{
    parse::{Parse, ParseStream, Result},
    visit_mut::{self, VisitMut},
    Block, Expr, ItemFn, Macro, Stmt, Token,
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
    current_section_id: Option<SectionId>,
}

impl ExpandBlock {
    fn expand_section_macro(&mut self, mac: &Macro) -> (Stmt, Option<SectionId>) {
        let body: SectionBody = match mac.parse_body() {
            Ok(body) => body,
            Err(err) => {
                let err = err.to_compile_error();
                return (syn::parse_quote!(#err), None);
            }
        };

        let name = &body.name;
        let block = &body.block;

        let section_id = self.next_section_id;
        let ancestors = if let Some(parent) = self.current_section_id {
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

        (
            syn::parse_quote! {
                if rye::_internal::is_target(#section_id) #block
            },
            Some(section_id),
        )
    }
}

impl VisitMut for ExpandBlock {
    fn visit_stmt_mut(&mut self, item: &mut Stmt) {
        let section_id = match item {
            Stmt::Expr(Expr::Macro(expr_macro)) | Stmt::Semi(Expr::Macro(expr_macro), _) => {
                if expr_macro.mac.path.is_ident("section") {
                    let (expanded, section_id) = self.expand_section_macro(&expr_macro.mac);
                    mem::replace(item, expanded);
                    section_id
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(section_id) = section_id {
            let prev = self.current_section_id.replace(section_id);
            visit_mut::visit_stmt_mut(self, item);
            self.current_section_id = prev;
        } else {
            visit_mut::visit_stmt_mut(self, item);
        }
    }
}

#[inline]
pub(crate) fn expand(item: &mut ItemFn) -> Vec<Section> {
    let mut expand = ExpandBlock {
        sections: IndexMap::new(),
        next_section_id: 0,
        current_section_id: None,
    };
    expand.visit_block_mut(&mut *item.block);
    expand.sections.into_iter().map(|(_k, v)| v).collect()
}
