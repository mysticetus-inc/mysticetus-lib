#![allow(dead_code, unreachable_code)] // TODO: remove after rebuilding
use proc_macro::TokenStream;

pub fn derive(parsed: ParsedTokens) -> syn::Result<TokenStream> {
    let unit = syn::TypeTuple {
        paren_token: Default::default(),
        elems: Default::default(),
    };

    let n_arities = parsed.generic_idents.len();
    let unit_vec = vec![&unit; n_arities];

    let mut indexes = Vec::with_capacity(n_arities);
    indexes.extend((0..n_arities).map(syn::Index::from));

    let mut dst_tokens = TokenStream::new();

    for (arity, arity_tokens) in parsed.per_arity_iter() {
        for n_units in 0..arity {
            let units = &unit_vec[..n_units];

            dst_tokens.extend(arity_tokens.impl_with_units(units, &indexes));
        }
    }

    Ok(dst_tokens)
}

pub struct ParsedTokens {
    trait_ident: syn::Type,
    generic_idents: Vec<syn::Ident>,
}

#[derive(Clone, Copy)]
struct PerArityTokens<'a> {
    trait_ident: &'a syn::Type,
    generic_idents: &'a [syn::Ident],
}

impl syn::parse::Parse for ParsedTokens {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ident = input.parse::<syn::Type>()?;

        input.parse::<syn::Token![;]>()?;

        let mut generic_idents = Vec::with_capacity(12);

        let p = syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(input)?;

        generic_idents.extend(p.into_iter());

        Ok(Self {
            trait_ident,
            generic_idents,
        })
    }
}

impl ParsedTokens {
    fn per_arity_iter(&self) -> impl ExactSizeIterator<Item = (usize, PerArityTokens<'_>)> {
        // start at one, since a primary key needs at least 1 component
        (1..self.generic_idents.len()).map(|end_at| {
            let toks = PerArityTokens {
                trait_ident: &self.trait_ident,
                generic_idents: &self.generic_idents[..end_at],
            };

            (end_at, toks)
        })
    }
}

impl PerArityTokens<'_> {
    fn impl_with_units(
        self,
        units: &[&syn::TypeTuple],
        full_indexes: &[syn::Index],
    ) -> TokenStream {
        let n_generics = match self.generic_idents.len().checked_sub(units.len()) {
            Some(n) => n,
            None => panic!("{} - {} ???", self.generic_idents.len(), units.len()),
        };

        let to_key_indexes = full_indexes[..n_generics].iter();
        let to_key_with_indexes = full_indexes[..n_generics].iter();
        let trait_ident = self.trait_ident;

        let generics = &self.generic_idents[..n_generics];

        let generics_def = generics.iter();
        let generics_self_ty = generics.iter();
        let generics_where = generics.iter();

        TokenStream::from(quote::quote! {
            impl< #( #generics_def, )* > #trait_ident for ( #( #generics_self_ty,)* #(#units),* )
            where
                #( #generics_where: crate::IntoSpanner, )*
            {
                #[inline]
                fn to_key(self) -> crate::__private::protobuf::ListValue {
                    crate::__private::protobuf::ListValue {
                        values: vec![
                            #(
                                crate::IntoSpanner::into_value(self.#to_key_indexes).into_protobuf(),
                            )*
                        ]
                    }
                }

                #[inline]
                fn to_key_with(self, final_value: crate::Value) -> crate::__private::protobuf::ListValue {
                    crate::__private::protobuf::ListValue {
                        values: vec![
                            #(
                                crate::IntoSpanner::into_value(self.#to_key_with_indexes).into_protobuf(),
                            )*
                            final_value.into_protobuf(),
                        ]
                    }
                }
            }
        })
    }
}
