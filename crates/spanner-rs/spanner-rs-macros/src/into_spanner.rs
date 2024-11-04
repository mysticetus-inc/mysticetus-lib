use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::DeriveInput;

pub fn derive(_inp: EnumInput) -> syn::Result<TokenStream> {
    todo!()
}

#[derive(Debug)]
pub struct EnumInput {
    variants: Vec<String>,
}

impl FromDeriveInput for EnumInput {
    fn from_derive_input(_input: &DeriveInput) -> darling::Result<Self> {
        todo!()
    }
}

impl syn::parse::Parse for EnumInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _inp = syn::DeriveInput::parse(input)?;
        todo!()
    }
}
