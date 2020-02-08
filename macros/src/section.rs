use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub(crate) type SectionId = u64;

pub(crate) struct Section {
    pub(crate) id: SectionId,
    pub(crate) name: syn::Expr,
    pub(crate) ancestors: Vec<SectionId>,
    pub(crate) children: Vec<SectionId>,
}

impl ToTokens for Section {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let name = &self.name;
        let is_leaf = self.children.is_empty();
        let ancestors = &self.ancestors;
        tokens.append_all(&[quote! {
            rye::_internal::Section::new(
                #id,
                #name,
                #is_leaf,
                rye::_internal::phf_set!(#(#ancestors),*)
            )
        }]);
    }
}
