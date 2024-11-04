#![allow(dead_code, unreachable_code, unused_variables)] // TODO: remove after rebuilding

use std::cmp::Ordering;

use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

use self::field::{BaseFieldOpts, FieldOpts, PkIndex};
use crate::ident_str::IdentStr;

mod cols;
mod field;
mod pk;

pub fn derive(mut table: TableInput) -> syn::Result<proc_macro::TokenStream> {
    table.validate_input()?;

    let cols::Cols {
        table_cols,
        col_defs_module,
    } = cols::derive_cols(&mut table)?;

    let table_impl = derive_table(&mut table, table_cols)?;

    let pk_impl = pk::derive_pk(&mut table)?;

    Ok(proc_macro::TokenStream::from(quote::quote! {
        #table_impl
        #pk_impl
        #col_defs_module
    }))
}

#[derive(Debug)]
pub struct TableInput {
    vis: syn::Visibility,
    ident: IdentStr,
    generics: syn::Generics,

    fields: Vec<FieldOpts>,
    table: Option<syn::LitStr>,
    pk_type: syn::Ident,
    column_module: syn::Ident,
}

impl syn::parse::Parse for TableInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = DeriveInput::parse(input)?;

        Self::from_input(input).map_err(syn::Error::from)
    }
}

impl TableInput {
    fn from_input(input: DeriveInput) -> darling::Result<Self> {
        /// Inner type to handle the heavy w.r.t data parsing
        #[derive(FromDeriveInput)]
        #[darling(attributes(spanner), forward_attrs(cfg), supports(struct_named))]
        struct RawStructInput {
            data: darling::ast::Data<darling::util::Ignored, FieldOpts>,
            #[darling(default)]
            table: Option<syn::LitStr>,
            #[darling(default)]
            pk_type: Option<syn::Ident>,
            #[darling(default)]
            column_module: Option<syn::Ident>,
        }

        let RawStructInput {
            data,
            table,
            pk_type,
            column_module,
        } = RawStructInput::from_derive_input(&input)?;

        // bail if we find enum data
        let fields = match data {
            darling::ast::Data::Struct(fields) => fields.fields,
            darling::ast::Data::Enum(_) => return Err(darling::Error::unsupported_shape("Enum")),
        };

        let pk_type = pk_type.unwrap_or_else(|| match table {
            Some(ref table_name) => quote::format_ident!("{}Pk", table_name.value()),
            None => quote::format_ident!("{}Pk", input.ident),
        });

        let column_module = column_module.unwrap_or_else(|| match table.as_ref() {
            Some(table) => quote::format_ident!("{}", table.value().to_case(Case::Snake)),
            None => quote::format_ident!("{}", input.ident.to_string().to_case(Case::Snake)),
        });

        Ok(Self {
            vis: input.vis,
            ident: IdentStr::from(input.ident),
            generics: input.generics,
            fields,
            pk_type,
            column_module,
            table,
        })
    }

    fn iter_pk_fields(&mut self) -> impl Iterator<Item = (&PkIndex, &mut BaseFieldOpts)> + '_ {
        self.fields.iter_mut().filter_map(FieldOpts::as_pk_mut)
    }

    fn validate_input(&mut self) -> syn::Result<()> {
        self.check_pks()?;

        Ok(())
    }

    fn check_pks(&mut self) -> syn::Result<()> {
        let mut pk_indexes = Vec::with_capacity(16);

        for (pk_index, field) in self.iter_pk_fields() {
            pk_indexes.push((pk_index, field));
        }

        if pk_indexes.is_empty() {
            return Err(syn::Error::new(
                self.ident.ident().span(),
                "no primary key columns specified, 1 or more is required",
            ));
        }

        // no unstable sort here, that way the user laid-out field order is implicitely secondary to
        // the indexes
        pk_indexes.sort_by_key(|(pk, _)| pk.index);

        for (expected, (pk, field)) in pk_indexes.into_iter().enumerate() {
            let message = match expected.cmp(&pk.index) {
                Ordering::Equal => continue,
                Ordering::Greater => format!("found duplicate pk index '{}'", pk.index),
                Ordering::Less => format!(
                    "pk indices aren't incremental, found {}, expected {}",
                    pk.index, expected
                ),
            };

            return Err(syn::Error::new(field.base.ident().span(), message));
        }

        Ok(())
    }
}

fn derive_table(info: &mut TableInput, table_cols: Vec<TokenStream>) -> syn::Result<TokenStream> {
    let TableInput {
        vis,
        ident,
        generics,
        fields,
        table,
        pk_type,
        ..
    } = info;

    let table_ident = &ident.ident();
    let default_table_name = &ident.literal();

    let n_fields = fields.len();
    let col_idents = fields
        .iter_mut()
        .map(|field| field.base.ident())
        .collect::<Vec<_>>();

    let to_row_body = to_row_body(fields);
    let from_row_body = from_row_body(fields);

    let col_idents1 = col_idents.iter().map(|i| i.as_ref());
    let col_idents2 = col_idents.iter().map(|i| i.as_ref());

    let indices = &(0..n_fields)
        .map(|idx| syn::LitInt::new(itoa::Buffer::new().format(idx), Span::call_site()))
        .collect::<Vec<_>>();

    let table_name = table.as_ref().unwrap_or(&default_table_name);

    let pk_fields = fields
        .iter_mut()
        .filter_map(FieldOpts::as_pk_mut)
        .map(|(_, field)| field.base.ident())
        .collect::<Vec<_>>();

    let pk_fields1 = pk_fields.iter().map(|i| i.as_ref());
    let pk_fields2 = pk_fields.iter().map(|i| i.as_ref());

    let unit: syn::Type = syn::parse_str("()")?;
    let empty_pk_units = (0..pk_fields.len()).map(|_| &unit);

    Ok(quote! {
        #[automatically_derived]
        impl #table_ident #generics {
            pub fn empty_key() -> #pk_type< #(#empty_pk_units,)* > {
                Default::default()
            }
        }

        #[automatically_derived]
        impl spanner_rs::Table for #table_ident #generics {
            const NAME: &'static str = #table_name;

            const COLS: &'static [&'static dyn ::spanner_rs::column::Column<Self>] = &[
                #(&#table_cols),*
            ];

            type PrimaryKey = #pk_type;

            fn to_row(self) -> ::core::result::Result<::spanner_rs::Row, ::spanner_rs::error::ConvertError> {
                #to_row_body
            }

            fn from_row(
                fields: &[::spanner_rs::Field],
                mut row: ::spanner_rs::Row,
            ) -> ::std::result::Result<Self, ::spanner_rs::error::ConvertError> {
                #from_row_body
            }

            fn into_pk(self) -> Self::PrimaryKey {
                <Self::PrimaryKey as ::spanner_rs::PrimaryKey<Self>>::from_parts((
                    #(
                        self.#pk_fields1.into(),
                    )*
                ))
            }

            fn to_pk(&self) -> Self::PrimaryKey {
                <Self::PrimaryKey as ::spanner_rs::PrimaryKey<Self>>::from_parts((
                    #(
                        self.#pk_fields2.clone().into(),
                    )*
                ))
            }
        }
    })
}

fn to_row_body(cols: &mut [FieldOpts]) -> proc_macro2::TokenStream {
    let col_expr = cols.iter_mut().map(|field| {
        let ident = field.base.ident();
        match (field.with.as_ref(), field.with_serde_as.as_ref()) {
            (Some(wrap), _) => quote::quote!( builder.add_column( #wrap(self.#ident))?; ),
            (_, Some(_)) => quote::quote!( builder.serialize_column(self.#ident)?; ),
            (None, None) => quote::quote!( builder.add_column(self.#ident)?; ),
        }
    });

    quote::quote! {
        let mut builder = ::spanner_rs::Row::builder::<Self>();
        #( #col_expr )*
        Ok(builder.build())
    }
}

fn from_row_body(cols: &mut [FieldOpts]) -> proc_macro2::TokenStream {
    let col_exprs = cols.iter_mut().enumerate().map(|(index, field)| {
        let index = syn::LitInt::new(itoa::Buffer::new().format(index), Span::call_site());
        let ident = field.base.ident();
        let ty = &field.ty;
        match (field.with.as_ref(), field.with_serde_as.as_ref()) {
            (Some(ref wrapper), _) => quote::quote! {
                #ident: match <#wrapper as ::spanner_rs::FromSpanner>::from_field_and_value(
                    &fields[#index],
                    row.take(#index),
                ) {
                    Ok(v) => v.0,
                    Err(err) => return Err(err),
                }
            },
            (_, Some(_)) => quote::quote! {
                #ident: match ::spanner_rs::serde::ValueDeserializer::deserialize::<#ty>(&fields[#index], row.take(#index)) {
                    Ok(v) => v,
                    Err(err) => return Err(err.into()),
                }
            },
            (None, None) => quote::quote! {
                #ident: match ::spanner_rs::FromSpanner::from_field_and_value(
                    &fields[#index],
                    row.take(#index),
                ) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                }
            },
        }
    });

    quote::quote! {
        Ok(Self {
            #( #col_exprs, )*
        })
    }
}
