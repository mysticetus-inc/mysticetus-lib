#![feature(let_chains)]
#![allow(dead_code, unreachable_code)] // TODO: remove after rebuilding

#[macro_use]
extern crate darling;

use proc_macro::TokenStream;

mod ident_str;

mod database;
mod impl_pk_sealed;
mod into_spanner;
mod table;

#[proc_macro]
pub fn query(tokens: TokenStream) -> TokenStream {
    tokens
}

#[proc_macro]
pub fn database(tokens: TokenStream) -> TokenStream {
    match database::parse(tokens) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro_derive(Table, attributes(spanner))]
pub fn table_derive(tokens: TokenStream) -> TokenStream {
    match table::derive(syn::parse_macro_input!(tokens as table::TableInput)) {
        Ok(tokens) => tokens,
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/*
#[proc_macro_derive(IntoSpanner, attributes(ty))]
pub fn into_spanner_derive(tokens: TokenStream) -> TokenStream {
    match table::derive(syn::parse_macro_input!(tokens as table::TableInput)) {
        Ok(tokens) => tokens,
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
*/

/// Internal helper macro for implementing [`spanner_rs::PkParts`] for tuples, with
/// impls for partial key variants containing ().
#[doc(hidden)]
#[proc_macro]
pub fn impl_pk_sealed(tokens: TokenStream) -> TokenStream {
    match impl_pk_sealed::derive(syn::parse_macro_input!(
        tokens as impl_pk_sealed::ParsedTokens
    )) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error().into(),
    }
}
