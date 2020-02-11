use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub(crate) type SectionId = u64;

pub(crate) struct Section {
    pub(crate) id: SectionId,
    pub(crate) name: syn::Expr,
    pub(crate) ancestors: Vec<SectionId>,
    pub(crate) children: Vec<SectionId>,
}

impl Section {
    pub(crate) fn map_entry(&self) -> SectionMapEntry<'_> {
        SectionMapEntry { section: self }
    }
}

pub(crate) struct SectionMapEntry<'a> {
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
