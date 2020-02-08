use std::{
    mem,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
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
    last_error: Option<Error>,
    is_async: bool,
}

impl ExpandBlock {
    fn throw_err(&mut self, err: Error) -> ! {
        self.last_error.replace(err);
        panic!("explicit panic");
    }

    fn expand_section_macro(&mut self, mac: &Macro) -> Stmt {
        let body: SectionBody = match mac.parse_body() {
            Ok(body) => body,
            Err(err) => self.throw_err(err),
        };

        let name = &body.name;
        let block = &body.block;

        let body = if self.is_async {
            quote::quote! {
                rye::_internal::with_section_async(&mut __section, async #block).await;
            }
        } else {
            quote::quote! {
                rye::_internal::with_section(&mut __section, || #block);
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
                #body
            }
        }}
    }

    fn expand(&mut self, block: &mut syn::Block) -> Result<()> {
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| self.visit_block_mut(block))) {
            if let Some(err) = self.last_error.take() {
                return Err(err);
            }
            resume_unwind(payload);
        }
        Ok(())
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
pub(crate) fn expand(item: &mut ItemFn) -> Result<()> {
    let is_async = item.sig.asyncness.is_some();
    ExpandBlock {
        last_error: None,
        is_async,
    }
    .expand(&mut *item.block)
}
