use std::rc::Rc;

use proc_macro2::Span;

use super::TableInput;
use super::field::{BaseFieldOpts, FieldOpts};

pub struct Cols {
    pub table_cols: Vec<proc_macro2::TokenStream>,
    pub col_defs_module: proc_macro2::TokenStream,
}

pub fn derive_cols(info: &mut TableInput) -> syn::Result<Cols> {
    let TableInput {
        vis,
        ident,
        column_module,
        generics,
        fields,
        table,
        pk_type,
    } = info;

    let table_ident = ident.ident();

    let mut col_type_impls = Vec::with_capacity(fields.len());
    let mut table_cols = Vec::with_capacity(fields.len());

    for (index, field) in fields.iter_mut().enumerate() {
        let (col_type_ident, col_impl) = gen_col_impl(index, &table_ident, field)?;
        col_type_impls.push(col_impl);

        table_cols.push(quote::quote!( #column_module::#col_type_ident ));
    }

    let col_defs_module = quote::quote! {
        #vis mod #column_module {
            use super::*;
            #(#col_type_impls)*
        }
    };

    Ok(Cols {
        table_cols,
        col_defs_module,
    })
}

fn gen_col_impl(
    index: usize,
    table: &syn::Ident,
    field: &mut FieldOpts,
) -> syn::Result<(Rc<syn::Ident>, proc_macro2::TokenStream)> {
    let field_ident = field.col_marker_ident();

    let BaseFieldOpts {
        base,
        pascal,
        screaming_snake,
        ty,
        is_option,
        rename,
        with,
        with_serde_as,
        marker_type,
    } = &mut **field;

    let field_lit = &pascal.literal();

    let lit_str_name = rename.as_ref().unwrap_or(&field_lit);

    let index_lit = syn::LitInt::new(itoa::Buffer::new().format(index), Span::call_site());

    let actual_ty = with_serde_as.as_ref().unwrap_or(ty);

    let nullable = syn::LitBool::new(*is_option, Span::call_site());

    let col_impl = quote::quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #field_ident;

        impl #field_ident {
            pub const COLUMN_INFO: ::spanner_rs::column::ColumnInfo = {
                use ::spanner_rs::{convert::SpannerEncode as _SpannerEncode, ty::SpannerType as _SpannerType};

                ::spanner_rs::column::ColumnInfo {
                    name: #lit_str_name,
                    index: #index_lit,
                    ty: <<#actual_ty as _SpannerEncode>::SpannerType as _SpannerType>::TYPE,
                    nullable: #nullable,
                }
            };
        }

        impl const ::spanner_rs::column::Column<super::#table> for #field_ident {
            fn info(&self) -> &'static ::spanner_rs::column::ColumnInfo {
                &Self::COLUMN_INFO
            }
        }
    };

    Ok((field_ident, col_impl))
}
