use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    MetaNameValue,
};

pub(crate) struct Args {
    pub(crate) values: Vec<MetaNameValue>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let values = Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated(input)?;
        Ok(Self {
            values: values.into_iter().collect(),
        })
    }
}
