use std::{
    mem,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    visit_mut::{self, VisitMut},
    Block, Expr, Ident, Macro, Stmt, Token,
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

struct ExpandBlock<'a> {
    last_error: Option<Error>,
    current_section_ident: &'a Ident,
}

impl ExpandBlock<'_> {
    fn throw_err(&mut self, err: Error) -> ! {
        self.last_error.replace(err);
        panic!("explicit panic");
    }

    fn expand_section_macro(&mut self, mac: &Macro) -> Stmt {
        let section = &*self.current_section_ident;

        let body: SectionBody = match mac.parse_body() {
            Ok(body) => body,
            Err(err) => self.throw_err(err),
        };

        let name = &body.name;
        let block = &body.block;

        syn::parse_quote! {{
            static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
                name: #name,
                file: file!(),
                line: line!(),
                column: column!(),
            };
            if let Some(__section) = #section.new_section(&SECTION) {
                let mut #section = __section;
                #block
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

impl VisitMut for ExpandBlock<'_> {
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
pub(crate) fn expand(block: &mut Block, current_section_ident: &Ident) -> Result<()> {
    ExpandBlock {
        last_error: None,
        current_section_ident,
    }
    .expand(block)
}
