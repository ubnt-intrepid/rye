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
    is_async: bool,
}

impl ExpandBlock {
    fn expand_section_macro(&mut self, mac: &Macro) -> Stmt {
        let body: SectionBody = match mac.parse_body() {
            Ok(body) => body,
            Err(err) => {
                let err = err.to_compile_error();
                return syn::parse_quote!(#err);
            }
        };

        let name = &body.name;
        let block = &body.block;

        let scoped = if self.is_async {
            quote::quote! {
                __section.scope_async(async #block).await;
            }
        } else {
            quote::quote! {
                __section.scope(|| #block);
            }
        };

        syn::parse_quote! {{
            static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                name: #name,
                file: file!(),
                line: line!(),
                column: column!(),
            };
            if let Some(mut __section) = rye::_internal::new_section(&SECTION) {
                #scoped
            }
        }}
    }
}

impl VisitMut for ExpandBlock {
    fn visit_stmt_mut(&mut self, item: &mut Stmt) {
        match item {
            Stmt::Expr(Expr::Macro(expr_macro)) | Stmt::Semi(Expr::Macro(expr_macro), _) => {
                if expr_macro.mac.path.is_ident("section") {
                    let expanded = self.expand_section_macro(&expr_macro.mac);
                    mem::replace(item, expanded);
                }
            }
            _ => (),
        }
        visit_mut::visit_stmt_mut(self, item);
    }
}

#[inline]
pub(crate) fn expand(item: &mut ItemFn) {
    let is_async = item.sig.asyncness.is_some();
    ExpandBlock { is_async }.visit_block_mut(&mut *item.block);
}
