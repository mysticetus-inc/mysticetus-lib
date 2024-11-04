use std::ops::{Bound, RangeBounds};
use std::rc::Rc;
use std::slice::SliceIndex;

use proc_macro2::TokenStream;

use super::TableInput;
use super::field::{BaseFieldOpts, FieldOpts, PkIndex};

pub(super) fn derive_pk(info: &mut TableInput) -> syn::Result<TokenStream> {
    let mut pk_info = DerivePkInfo::init(info);
    let base = pk_info.generate_base_impls();

    let pk_variants_iter =
        (0..=pk_info.pk_fields.len()).map(|up_to| pk_info.generate_partial_impl(up_to));

    Ok(quote::quote! {
        #base
        #(
            #pk_variants_iter
        )*
    })
}

struct DerivePkInfo<'a> {
    vis: &'a syn::Visibility,
    pk_type: &'a syn::Ident,
    table_ident: Rc<syn::Ident>,
    pk_fields: Vec<PkField<'a>>,
    tuple_indices: Vec<syn::Index>,
    generics: Vec<syn::Ident>,
    unit_ty: syn::Type,
}

type PkField<'a> = (&'a PkIndex, &'a mut BaseFieldOpts);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
enum PkFieldType {
    First,
    Intermediate,
    Last,
}

impl<'a> DerivePkInfo<'a> {
    fn init(info: &'a mut TableInput) -> Self {
        let TableInput {
            vis,
            ident,
            generics,
            fields,
            table,
            pk_type,
            ..
        } = info;

        let table_ident = ident.ident();

        let pk_fields = fields
            .iter_mut()
            .filter_map(FieldOpts::as_pk_mut)
            .collect::<Vec<_>>();

        let pk_types1 = pk_fields.iter().map(|(_, field)| &field.ty);
        let pk_types2 = pk_fields.iter().map(|(_, field)| &field.ty);

        let tuple_indices = (0..pk_fields.len())
            .map(syn::Index::from)
            .collect::<Vec<_>>();

        let generics = (0..pk_fields.len())
            .map(|index| quote::format_ident!("Pk{}", index))
            .collect::<Vec<_>>();

        Self {
            table_ident,
            pk_fields,
            tuple_indices,
            generics,
            unit_ty: syn::parse_str("()").expect("should never fail"),
            vis: &info.vis,
            pk_type: &info.pk_type,
        }
    }

    fn pk_field_type(&self, up_to: usize) -> PkFieldType {
        if up_to == 0 {
            PkFieldType::First
        } else if up_to < self.pk_fields.len() {
            PkFieldType::Intermediate
        } else {
            PkFieldType::Last
        }
    }

    fn pk_type_iter<'i, R>(&'i self, range: R) -> PkTypeIter<'i>
    where
        R: SliceIndex<[PkField<'a>], Output = [PkField<'a>]> + RangeBounds<usize>,
    {
        assert!(
            matches!(range.start_bound(), Bound::Unbounded | Bound::Included(0)),
            "type ranges should always begin with the first pk field type"
        );

        let subset = &self.pk_fields[range];

        PkTypeIter {
            unit: &self.unit_ty,
            units_remaining: self.pk_fields.len() - subset.len(),
            leading: subset.iter(),
        }
    }

    fn generate_base_impls(&self) -> proc_macro2::TokenStream {
        let table_ident = &*self.table_ident;
        let tuple_indices = &*self.tuple_indices;
        let generics = &*self.generics;
        let vis = self.vis;
        let pk_type = self.pk_type;

        let pk_types1 = self.pk_type_iter(..);
        let pk_types2 = self.pk_type_iter(..);
        let pk_types3 = self.pk_type_iter(..);

        let unit_iter1 = self.pk_type_iter(..0);
        let unit_iter2 = self.pk_type_iter(..0);

        quote::quote! {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #vis struct #pk_type <
                #(
                    #generics = #pk_types1,
                )*
            > (
                #(
                    pub #generics,
                )*
            );

            impl Default for #pk_type<#(#unit_iter1,)*> {
                #[inline(always)]
                fn default() -> Self {
                    Self(
                        #(#unit_iter2,)*
                    )
                }
            }

            #[automatically_derived]
            impl ::spanner_rs::pk::PrimaryKey<#table_ident> for #pk_type {
                type Parts = (
                    #(#pk_types3,)*
                );

                fn into_parts(self) -> Self::Parts {
                    (
                        #(self.#tuple_indices,)*
                    )
                }

                fn from_parts(parts: Self::Parts) -> Self {
                    Self(
                        #(parts.#tuple_indices,)*
                    )
                }
            }
        }
    }

    fn generate_builder_fn(
        &mut self,
        up_to: usize,
        pk_field_type: PkFieldType,
    ) -> Option<proc_macro2::TokenStream> {
        if pk_field_type == PkFieldType::Last {
            return None;
        }
        let pk_type = self.pk_type;

        let field = &mut self.pk_fields[up_to];

        let field_name = &field.1.base.ident();
        let field_ty = field.1.ty.clone();

        let indices = &self.tuple_indices[..up_to];

        let rem_units = self.pk_fields.len() - (up_to + 1);
        let rem_unit_iter = (0..rem_units).map(|_| &self.unit_ty);

        let curr_pk_types = self.pk_type_iter(..up_to);
        let next_pk_types = self.pk_type_iter(..up_to + 1);

        if pk_field_type == PkFieldType::First {
            let new_fn_ident = quote::format_ident!("new_{}", &**field_name);

            let next_pk_types_2 = self.pk_type_iter(..up_to + 1);
            let rem_unit_iter_2 = (0..rem_units).map(|_| &self.unit_ty);

            Some(quote::quote! {
                impl #pk_type<#(#curr_pk_types,)*> {
                    #[inline]
                    pub fn #field_name<K>(self, #field_name: K) -> #pk_type<#(#next_pk_types,)*>
                    where
                        K: ::core::convert::Into<#field_ty>
                    {
                        #pk_type(
                            ::core::convert::Into::into(::core::convert::Into::into(#field_name)),
                            #(#rem_unit_iter,)*
                        )
                    }

                    #[inline]
                    pub fn #new_fn_ident<K>(#field_name: K) -> #pk_type<#(#next_pk_types_2,)*>
                    where
                        K: ::core::convert::Into<#field_ty>
                    {
                        #pk_type(
                            ::core::convert::Into::into(::core::convert::Into::into(#field_name)),
                            #(#rem_unit_iter_2,)*
                        )
                    }
                }
            })
        } else {
            Some(quote::quote! {
                impl #pk_type<#(#curr_pk_types,)*> {

                    #[inline]
                    pub fn #field_name<K>(self, #field_name: K) -> #pk_type<#(#next_pk_types,)*>
                    where
                        K: ::core::convert::Into<#field_ty>
                    {
                        #pk_type(
                            #(self.#indices,)*
                            ::core::convert::Into::into(::core::convert::Into::into(#field_name)),
                            #(#rem_unit_iter,)*
                        )
                    }
                }
            })
        }
    }

    fn generate_partial_impl(&mut self, up_to: usize) -> proc_macro2::TokenStream {
        let pk_field_type = self.pk_field_type(up_to);
        // generate the builder fn first, so we dont hold onto any borrows
        let builder_fn = self.generate_builder_fn(up_to, pk_field_type);

        // if this is the first pk field, we can only do the builder fn (cant do impls over a pk
        // with only unit types)
        if pk_field_type == PkFieldType::First {
            return builder_fn.unwrap_or_default();
        }

        let table_ident = &*self.table_ident;
        let tuple_indices = &*self.tuple_indices;
        let generics = &*self.generics;
        let vis = self.vis;
        let pk_type = self.pk_type;

        // if this isnt the first or final key component, generate an impl for tuples with no units
        // (i.e an impl for both '(Pk1, Pk2, (), ())' and '(Pk1, Pk2,)' that have the same meaning)
        let no_unit_pk_impl = if pk_field_type == PkFieldType::Intermediate {
            // tweak the iterator to stop before yielding any unit types, to generate partial
            let mut pk_types_no_units = self.pk_type_iter(..up_to);
            pk_types_no_units.units_remaining = 0;

            Some(quote::quote! {
                #[automatically_derived]
                impl ::spanner_rs::pk::PartialPkParts<#table_ident> for ( #(#pk_types_no_units,)* ) { }
            })
        } else {
            None
        };

        let pk_types1 = self.pk_type_iter(..up_to);
        let pk_types2 = self.pk_type_iter(..up_to);
        let pk_types3 = self.pk_type_iter(..up_to);

        quote::quote! {
            #[automatically_derived]
            impl ::spanner_rs::pk::IntoPartialPkParts<#table_ident> for #pk_type<
                #(#pk_types1,)*
            > {
                type PartialParts = (#(#pk_types2,)*);

                fn into_partial_parts(self) -> Self::PartialParts {
                    (
                        #(self.#tuple_indices,)*
                    )
                }
            }

            #builder_fn

            #no_unit_pk_impl

            #[automatically_derived]
            impl ::spanner_rs::pk::PartialPkParts<#table_ident> for ( #(#pk_types3,)* ) { }

        }
    }
}

struct PkTypeIter<'a> {
    unit: &'a syn::Type,
    units_remaining: usize,
    leading: std::slice::Iter<'a, PkField<'a>>,
}

enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> quote::ToTokens for Either<A, B>
where
    A: quote::ToTokens,
    B: quote::ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Left(l) => l.to_tokens(tokens),
            Self::Right(r) => r.to_tokens(tokens),
        }
    }
}

impl<'a> Iterator for PkTypeIter<'a> {
    type Item = Either<&'a syn::Type, &'a syn::Ident>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, next)) = self.leading.next() {
            match next.with.as_ref() {
                Some(with) => return Some(Either::Right(with)),
                None => return Some(Either::Left(&next.ty)),
            }
        }

        self.units_remaining = self.units_remaining.checked_sub(1)?;
        Some(Either::Left(self.unit))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.units_remaining + self.leading.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for PkTypeIter<'_> {}
