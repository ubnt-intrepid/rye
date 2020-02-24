use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt as _};
use std::mem;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    visit_mut::{self, VisitMut},
    Attribute, Block, Expr, ExprAsync, ExprClosure, ExprForLoop, ExprLoop, ExprWhile, Item, ItemFn,
    Macro, MetaNameValue, Stmt, Token,
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

    // extract attributes
    let params = try_quote!(Params::from_attrs(&mut item.attrs));

    // expand section!()
    let mut expand = ExpandBlock {
        params: &params,
        sections: IndexMap::new(),
        next_section_id: 0,
        parent: None,
        in_loop: false,
        in_closure: false,
        in_async_block: false,
    };
    expand.visit_block_mut(&mut *item.block);
    let sections: Vec<_> = expand.sections.into_iter().map(|(_k, v)| v).collect();

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
    rye_path: syn::Path,
}

impl Params {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut rye_path = None;
        let mut errors: Option<Error> = None;

        attrs.retain(|attr| {
            if !attr.path.is_ident("rye") {
                return true;
            }

            if let Err(error) = (|| -> Result<()> {
                let param: MetaNameValue = attr.parse_args()?;
                if param.path.is_ident("rye_path") {
                    match param.lit {
                        syn::Lit::Str(ref lit) if rye_path.is_none() => {
                            rye_path.replace(lit.parse()?);
                            Ok(())
                        }
                        syn::Lit::Str(..) => {
                            Err(Error::new_spanned(&param, "duplicated parameter"))
                        }
                        lit => Err(Error::new_spanned(&lit, "required a string literal")),
                    }
                } else {
                    Err(Error::new_spanned(&param.path, "unknown parameter name"))
                }
            })() {
                if let Some(ref mut errors) = errors {
                    errors.combine(error);
                } else {
                    errors.replace(error);
                }
            }

            false
        });

        if let Some(errors) = errors {
            return Err(errors);
        }

        Ok(Self {
            rye_path: rye_path.unwrap_or_else(|| syn::parse_quote!(::rye)),
        })
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

struct ExpandBlock<'a> {
    params: &'a Params,
    sections: IndexMap<SectionId, Section>,
    next_section_id: SectionId,
    parent: Option<SectionId>,
    in_loop: bool,
    in_closure: bool,
    in_async_block: bool,
}

impl ExpandBlock<'_> {
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

        let rye_path = &self.params.rye_path;
        Ok((
            syn::parse_quote! {{
                let __section = #rye_path::_internal::enter_section(#section_id);
                if __section.enabled() #block
                __section.leave();
            }},
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

impl VisitMut for ExpandBlock<'_> {
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
            rye_path: &'a syn::Path,
        }

        impl ToTokens for SectionMapEntry<'_> {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let id = &self.section.id;
                let name = &self.section.name;
                let ancestors = &self.section.ancestors;
                let rye_path = &self.rye_path;
                tokens.append_all(&[quote! {
                    #id => #rye_path::_internal::Section {
                        name: #name,
                        ancestors: #rye_path::_internal::hashset!(#( #ancestors ),*),
                    }
                }]);
            }
        }

        let section_map_entries = self.sections.iter().map(|section| SectionMapEntry {
            section,
            rye_path: &self.params.rye_path,
        });
        let leaf_section_ids = self.sections.iter().filter_map(|section| {
            if section.children.is_empty() {
                Some(section.id)
            } else {
                None
            }
        });

        let ident = &self.item.sig.ident;
        let rye_path = &self.params.rye_path;

        let test_fn: syn::Expr = if self.item.sig.asyncness.is_some() {
            let local = self.args.local;
            let constructor = if local {
                quote!(new_local)
            } else {
                quote!(new)
            };
            syn::parse_quote!(#rye_path::_internal::TestFn::Async {
                f: || #rye_path::_internal::TestFuture::#constructor(super::#ident()),
                local: #local,
            })
        } else {
            syn::parse_quote!(#rye_path::_internal::TestFn::Blocking {
                f: || #rye_path::_internal::test_result(super::#ident()),
            })
        };

        tokens.append_all(vec![quote! {
            pub(crate) mod #ident {
                use super::*;

                struct __registration(());

                impl #rye_path::_internal::Registration for __registration {
                    fn register(&self, __registry: &mut dyn #rye_path::_internal::Registry) -> #rye_path::_internal::Result<(), #rye_path::_internal::RegistryError> {
                        __registry.add_test(#rye_path::_internal::Test {
                            desc: #rye_path::_internal::TestDesc {
                                module_path: #rye_path::_internal::module_path!(),
                                sections: #rye_path::_internal::hashmap! { #( #section_map_entries, )* },
                                leaf_sections: #rye_path::_internal::vec![ #( #leaf_section_ids ),* ],
                            },
                            test_fn: #test_fn,
                        })?;
                        #rye_path::_internal::Result::Ok(())
                    }
                }

                #rye_path::__annotate_test_case! {
                    pub(crate) const __REGISTRATION: &dyn #rye_path::_internal::Registration = &__registration(());
                }
            }
        }]);
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
}
