/// Defines a struct as a spanner table, and implements all of the nessecary traits.
///
/// Should be used within its own module, as names will clash if 2 invocations of
/// [`row!`] are called in the same module, unless certain options are used (to override the
/// defaults)
///
/// Notes + Limitations:
///     - functions/modules specified in `encode_with`/`decode_with`/`with` field options need to be
///       single identifiers (i.e can't be a path like `crate::util::...`).
#[macro_export]
macro_rules! row {
    // adapted (heavily) from the diesel::table macro
    ($($tokens:tt)*) => {
        $crate::__parse_row! {
            tokens = [$($tokens)*],
            imports = [],
            meta = [],
            unprocessed_spanner_meta = [],
            row_ident = unknown,
            row_vis = unknown,
            table_name = [],
            generics = [],
            pk_name = [PrimaryKey],
            pks = [],
        }
    }
}

crate::row! {
    #[derive(Debug, Clone, PartialEq)]
    #[spanner(table = "Test")]
    pub struct TestRow<T> {
        #[spanner(generic, pk = 1)]
        pub a_field: T,
        #[spanner(pk = 2)]
        pub non_generic: u32,
    }
}

pub mod scope_values {
    crate::row! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        pub struct ScopeValues {
            pub scope: Box<str>,
            pub values: Vec<Box<str>>,
        }
    }
}

const _: () = {
    #[allow(unused)]
    const fn assert_insertable<T: crate::Table>(_: &T) {}
    assert_insertable::<TestRow<&str>>(&TestRow {
        a_field: "",
        non_generic: 0,
    });
};

#[macro_export]
#[doc(hidden)]
macro_rules! __invalid_row_syntax {
    ($inside:literal $(,)? $($tokens:tt)+) => {
        /*
        debug_macro::debug_macro! {
           const INSIDE: &str = $inside;
           $($tokens)+
        }
        */
        #[cfg(feature = "debug-table-macro")]
        compile_error!(concat!(
            "Invalid `row!` syntax inside ",
            $inside,
            " '",
            $(stringify!($tokens),)+
            "'"
        ));

        // #[cfg(not(feature = "debug-table-macro"))]
        // $crate::__invalid_row_syntax!($inside)
    };
    ($inside:literal  $(,)?) => {
        compile_error!("Invalid `row!` syntax")
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_row {
    // Found an import
    (
        tokens = [use $($import:tt)::+; $($rest:tt)*],
        imports = [$($imports:tt)*],
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = [$($imports)* use $($import)::+;],
            $($args)*
        }
    };
    // we found a container-level spanner attribute
    (
        tokens = [#[spanner($($attrs:tt)*)] $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$($prev:tt)*],
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($prev:tt)* $($attrs)*],
            $($args)*
        }
    };

    // Found table = "" attribute, override whatever we had before
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? table = $table_name:literal $($rest_unprocessed:tt)*],
        row_ident = $row_ident:tt,
        row_vis = $row_vis:tt,
        table_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = $tokens,
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($rest_unprocessed)*],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = [$table_name],
            $($args)*
        }
    };
    // Found bare table attribute, override whatever we had before
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? table $($rest_unprocessed:tt)*],
        row_ident = $row_ident:tt,
        row_vis = $row_vis:tt,
        table_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = $tokens,
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($rest_unprocessed)*],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = [__use_struct_ident],
            $($args)*
        }
    };

    // Found pk_name attribute, override whatever we had before
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? pk_name = $pk_name:ident $($rest_unprocessed:tt)*],
        row_ident = $row_ident:tt,
        row_vis = $row_vis:tt,
        table_name = $table_name:tt,
        generics = [],
        pk_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = $tokens,
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($rest_unprocessed)*],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = $table_name,
            generics = [],
            pk_name = [$pk_name],
            $($args)*
        }
    };

    // Meta item other than sql_name, attach it to the table struct
    (
        tokens = [#$new_meta:tt $($rest:tt)*],
        imports = $imports:tt,
        meta = [$($meta:tt)*],
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = [$($meta)* #$new_meta],
            $($args)*
        }
    };
    // do a specific error on an unknown spanner option
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? $unknown:ident $($ignored:tt)*]
        $($args:tt)*
    ) => {
        compile_error!(concat!("unknown spanner::row! option `", stringify!($unknown), "`"));
    };
    // Bare #[spanner(table)] was used, so we need to stringify the struct identifier to
    // make the table name.
    (
        tokens = [$row_vis:vis struct $row_ident:ident $($rest:tt)* ],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)?],
        row_ident = $ignore:tt,
        row_vis = $ignore2:tt,
        table_name = [__use_struct_ident],
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = [stringify!($row_ident)],
            $($args)*
        }
    };
    // Found the table/struct definition
    (
        tokens = [$row_vis:vis struct $row_ident:ident $($rest:tt)* ],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)?],
        row_ident = $ignore:tt,
        row_vis = $ignore2:tt,
        table_name = $table_name:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = $table_name,
            $($args)*
        }
    };

    // parse generics
    (
        tokens = [< $($generics:tt),+ > $($rest:tt)* ],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)?],
        row_ident = $row_ident:tt,
        row_vis = $row_vis:tt,
        table_name = $table_name:tt,
        generics = $ignore3:tt,
        $($args:tt)*
    ) => {

        $crate::__parse_row! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [],
            row_ident = $row_ident,
            row_vis = $row_vis,
            table_name = $table_name,
            generics = [$($generics)*],
            $($args)*
        }
    };

    // Parse the columns
    (
        tokens = [{$($columns:tt)*}],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [],
        row_ident = $row_ident:tt,
        row_vis = $row_vis:tt,
        table_name = $table_name:tt,
        generics = $generics:tt,
        pk_name = [$pk_name:ident],
        pks = [],
    ) => {
        $crate::__parse_columns! {
            tokens = [$($columns)*],
            next_column_index = [0],
            row = {
                imports = $imports,
                meta = $meta,
                row = $row_ident,
                row_vis = $row_vis,
                table_name = $table_name,
            },
            columns = [],
            generics = $generics,
            pks = [],
            pk_name = [$pk_name],
        }
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        $crate::__invalid_row_syntax!("parse_row" $($tokens)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_columns {
    // No column being parsed, start a new one.
    // Attempt to capture the type as separate tokens if at all possible.
    (
        tokens = [
            $(#$meta:tt)*
            $field_vis:vis $field:ident: $($ty:tt)::* $(<$($ty_params:tt)::*>)*,
            $($rest:tt)*
        ],
        next_column_index = [$next_col_idx:expr],
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [$(#$meta)*],
                spanner_args = [],
                field = $field,
                field_vis = $field_vis,
                field_name = [__UNSPECIFIED],
                ty = ($($ty)::* $(<$($ty_params)::*>)*),
                meta = [],
                encode_with = unknown,
                decode_with = unknown,
                column_index = $next_col_idx,
                pk_index = unknown,
                generic = no,
            },
            tokens = [$($rest)*],
            next_column_index = [$next_col_idx + 1],
            $($args)*
        }
    };

    // No column being parsed, start a new one. Couldn't keep the `ty` separate.
    (
        tokens = [
            $(#$meta:tt)*
            $field_vis:vis $field:ident: $ty:ty,
            $($rest:tt)*
        ],
        next_column_index = [$next_col_idx:expr],
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [$(#$meta)*],
                spanner_args = [],
                field = $field,
                field_vis = $field_vis,
                field_name = [__UNSPECIFIED],
                ty = ($ty),
                meta = [],
                encode_with = unknown,
                decode_with = unknown,
                column_index = $next_col_idx,
                pk_index = unknown,
                generic = no,
            },
            tokens = [$($rest)*],
            next_column_index = [$next_col_idx + 1],
            $($args)*
        }
    };

    // #[spanner(...)] meta item
    (
        current_column = {
            unchecked_meta = [ #[spanner( $($spanner_args:tt)* )] $($meta:tt)*],
            spanner_args = [$($existing_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [$($meta)*],
                spanner_args = [$($spanner_args)* $($existing_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                $($current_column)*
            },
            $($args)*
        }
    };

    // Meta item other than #[spanner(...)]
    (
        current_column = {
            unchecked_meta = [#$new_meta:tt $($unchecked_meta:tt)*],
            spanner_args = $spanner_args:tt,
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = [$($meta:tt)*],
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [$($unchecked_meta)*],
                spanner_args = $spanner_args,
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = [$($meta)* #$new_meta],
                $($current_column)*
            },
            $($args)*
        }
    };
    // parsing spanner(generic)
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? generic $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = $decode_with:tt,
            column_index = $col_idx:tt,
            pk_index = $pk_index:tt,
            generic = $ignore2:tt,
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
                pk_index = $pk_index,
                generic = yes,
            },
            $($args)*
        }
    };
    // parsing spanner field rename
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? rename = $rename_as:ident $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $ignore:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = [$rename_as],
                ty = $ty,
                meta = $meta,
                $($current_column)*
            },
            $($args)*
        }
    };
    // parsing spanner pk
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? pk = $pk:literal $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = $decode_with:tt,
            column_index = $col_idx:tt,
            pk_index = $ignore:tt,
            generic = $generic:tt,
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
                pk_index = $pk,
                generic = $generic,
            },
            $($args)*
        }
    };

    // parsing spanner with, similar to how serdes 'with' combines deserialize_with and serialize_with
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? with = $with:ident $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $ignore:tt,
            decode_with = $ignore2:tt,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = [<$with<_> as $crate::SpannerEncode>::encode],
                decode_with = [<$with<_> as $crate::FromSpanner>::from_value],
                $($current_column)*
            },
            $($args)*
        }
    };

    // parsing spanner encode_with, as a single ident
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? encode_with = $encode_with:ident $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $ignore:tt,
            $($current_column:tt)*$(,)?
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = [$encode_with],
                $($current_column)*
            },
            $($args)*
        }
    };

    // parsing spanner decode_with
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? decode_with = $decode_with:ident $($rest_spanner_args:tt)*],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = $ignore:tt,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [$($rest_spanner_args)*],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = [$decode_with],
                $($current_column)*
            },
            $($args)*
        }
    };

    // setting default 'encode_with'
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)?],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = unknown,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = [$crate::__macro_internals::to_spanner],
                $($current_column)*
            },
            $($args)*
        }
    };

    // setting default 'decode_with'
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)?],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = unknown,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            current_column = {
                unchecked_meta = [],
                spanner_args = [],
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = [$crate::__macro_internals::from_spanner],
                $($current_column)*
            },
            $($args)*
        }
    };
    // Done parsing a non-pk column
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = $decode_with:tt,
            column_index = $col_idx:tt,
            pk_index = unknown,
            generic = $generic:tt,
        },
        tokens = $tokens:tt,
        next_column_index = $next_col_idx:tt,
        row = $row:tt,
        columns = [$($columns:tt,)*],
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            tokens = $tokens,
            next_column_index = $next_col_idx,
            row = $row,
            columns = [$($columns,)* {
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
                generic = $generic,
            },],
            $($args)*
        }
    };

    // Done parsing a pk column
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [],
            field = $field:tt,
            field_vis = $field_vis:tt,
            field_name = $field_name:tt,
            ty = $ty:tt,
            meta = $meta:tt,
            encode_with = $encode_with:tt,
            decode_with = $decode_with:tt,
            column_index = $col_idx:tt,
            pk_index = $pk_index:literal,
            generic = $generic:tt,
        },
        tokens = $tokens:tt,
        next_column_index = $next_col_idx:tt,
        row = $row:tt,
        columns = [$($columns:tt,)*],
        generics = $generics:tt,
        pks = [$($existing_pks:tt)*],
        pk_name = $pk_name:tt,
    ) => {
        $crate::__parse_columns! {
            tokens = $tokens,
            next_column_index = $next_col_idx,
            row = $row,
            columns = [$($columns,)* {
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
                generic = $generic,
            },],
            generics = $generics,
            pks = [$($existing_pks)* ($field, $ty, $pk_index, $generic),],
            pk_name = $pk_name,
        }
    };

    // Done parsing all columns
    (
        tokens = [],
        next_column_index = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__row_impls!($($args)*);
    };

    ($($tokens:tt)*) => {
        $crate::__invalid_row_syntax!("parse_columns" $($tokens)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_col {
    (
        field = $field:ident,
        field_ty = ($($field_ty:tt)*),
        field_name = [$field_name:expr],
        col_index = $col_index:expr,
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $field;

        impl $crate::queryable::new::Column for $field {
            const NAME: &'static str = $field_name;
            type Type = $($field_ty)*;
            type Index = typenum::U<{ $col_index }>;
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_columns {
    (
        $({
            field = $field:ident,
            field_name = $field_name:tt,
            ty = ($($column_ty:tt)*),
            column_index = $col_idx:expr,
        }),*
        $(,)?
    ) => {{
        $crate::__macro_internals::generic_array::GenericArray::from_array([
            $(
                $crate::__impl_columns! {
                    field = $field,
                    field_name = $field_name,
                    ty = ($($column_ty)*),
                    column_index = $col_idx,
                },
            )*
        ])
    }};
    (
        field = $field:ident,
        field_name = [__UNSPECIFIED],
        ty = ($($column_ty:tt)*),
        column_index = $col_idx:expr,
    ) => {{
        $crate::__macro_internals::paste! {
            $crate::column::Column::new::<<$($column_ty)* as $crate::SpannerEncode>::SpannerType>($col_idx, stringify!([<$field:camel>]))
        }
    }};
    (
        field = $field:ident,
        field_name = [$field_name:ident],
        ty = ($($column_ty:tt)*),
        column_index = $col_idx:expr,
    ) => {{
        $crate::column::Column::new::<<$($column_ty)* as $crate::SpannerEncode>::SpannerType>($col_idx, stringify!($field_name))
    }};
    ($($t:tt)*) => {
        $crate::__invalid_row_syntax!("__impl_columns" $($t)*)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __row_impls {
    (
        row = {
            imports = [$($imports:tt)*],
            meta = [$($meta:tt)*],
            row = $row:ident,
            row_vis = $row_vis:vis,
            table_name = [$($table_name:expr)?],
        },
        columns = [
            $(
                {
                    field = $field:ident,
                    field_vis = $field_vis:vis,
                    field_name = $field_name:tt,
                    ty = ($($column_ty:tt)*),
                    meta = [$($column_metas:tt)*],
                    encode_with = [$($encode_with:tt)*],
                    decode_with = [$($decode_with:tt)*],
                    column_index = $col_idx:expr,
                    generic = $generic:tt,
                }
            ),+
            $(,)?
        ],
        generics = [$($generics:tt)*],
        pks = [$(($pk_field:ident, ($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],
        pk_name = [$pk_name:ident],
    ) => {
        $($meta)*
        $row_vis struct $row <$($generics)*> {
            $(
                $($column_metas)*
                $field_vis $field: $($column_ty)*,
            )+
        }

        impl<$($generics)*> $crate::queryable::Row for $row <$($generics)*>
        where
            $($generics: $crate::SpannerEncode,)*
        {
            type NumColumns = $crate::__macro_internals::typenum::U<{ <[()]>::len(&[$($crate::__replace_with_unit!($field),)*]) }>;
            type ColumnName = &'static str;

            const COLUMNS: $crate::__macro_internals::generic_array::GenericArray<$crate::column::Column<'static>, Self::NumColumns> = $crate::__impl_columns! {
                $({
                    field = $field,
                    field_name = $field_name,
                    ty = ($($column_ty)*),
                    column_index = $col_idx,
                }),*
            };
        }


        impl<$($generics)*> $crate::queryable::Queryable for $row <$($generics)*>
        where
            Self: $crate::queryable::Row,
            $($generics: $crate::FromSpanner,)*
        {
            fn from_row(mut row: $crate::results::RawRow<'_, Self::NumColumns>) -> $crate::Result<Self> {
                Ok(Self {
                    $(
                        $field: row.decode_at_index($col_idx, $($decode_with)*)?,
                    )*
                })
            }
        }

        /*
        pub mod columns {
            $(
                $crate::__impl_col! {
                    field = $field,
                    field_ty = ($($column_ty)*),
                    field_name = [$field_name],
                    col_index = $col_idx,
                }
            )+
        }
        */

        $crate::__impl_table! {
            row = {
                imports = [$($imports)*],
                meta = [$($meta)*],
                row = $row,
                row_vis = $row_vis,
                table_name = [$($table_name)?],
            },
            columns = [
                $(
                    {
                        field = $field,
                        field_vis = $field_vis,
                        field_name = $field_name,
                        ty = ($($column_ty)*),
                        meta = [$($column_metas)*],
                        encode_with = [$($encode_with)*],
                        decode_with = [$($decode_with)*],
                        column_index = $col_idx,
                        generic = $generic,
                    }
                ),+
            ],
            pks = [$(($pk_field, ($($pk_type)*), $pk_index, $pk_generic)),*],
            pk_name = [$pk_name],
            generics = [$($generics)*],
        }

        $crate::__impl_row_builder! {
            row = {
                imports = [$($imports)*],
                meta = [$($meta)*],
                row = $row,
                row_vis = $row_vis,
                table_name = [$($table_name)?],
            },
            columns = [
                $(
                    {
                        field = $field,
                        field_vis = $field_vis,
                        field_name = $field_name,
                        ty = ($($column_ty)*),
                        meta = [$($column_metas)*],
                        encode_with = [$($encode_with)*],
                        decode_with = [$($decode_with)*],
                        column_index = $col_idx,
                        generic = $generic,
                    }
                ),+
            ],
            pks = [$(($pk_field, ($($pk_type)*), $pk_index, $pk_generic)),*],
            pk_name = [$pk_name],
            generics = [$($generics)*],
        }
    };
    ($($t:tt)*) => {
        $crate::__invalid_row_syntax!("__row_impls" $($t)*);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_table {
    // if there's no table name, expand to nothing
    (
        row = {
            imports = $ignore_imports:tt,
            meta = $ignore_meta:tt,
            row = $ignore_row:ident,
            row_vis = $ignore_row_vis:vis,
            table_name = [],
        },
        columns = [
            $(
                {
                    field = $field:ident,
                    field_vis = $field_vis:vis,
                    field_name = [$field_name:expr],
                    ty = ($($column_ty:tt)*),
                    meta = [$($column_metas:tt)*],
                    encode_with = [$($encode_with:tt)*],
                    decode_with = [$($decode_with:tt)*],
                    column_index = $col_idx:expr,
                    generic = $generic:tt,
                }
            ),+
            $(,)?
        ],
        pks = $ignore_pks:tt,
        pk_name = $ignore_pk_name:tt,
        generics = $ignore_generics:tt,
    ) => {};
    // if there is, impl Table
    (
        row = {
            imports = [$($imports:tt)*],
            meta = [$($meta:tt)*],
            row = $row:ident,
            row_vis = $row_vis:vis,
            table_name = [$table_name:expr],
        },
        columns = [
            $(
                {
                    field = $field:ident,
                    field_vis = $field_vis:vis,
                    field_name = [$field_name:expr],
                    ty = ($($column_ty:tt)*),
                    meta = [$($column_metas:tt)*],
                    encode_with = [$($encode_with:tt)*],
                    decode_with = [$($decode_with:tt)*],
                    column_index = $col_idx:expr,
                    generic = $generic:tt,
                }
            ),+
            $(,)?
        ],
        pks = [$(($pk_field:ident, ($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],
        pk_name = [$pk_name:ident],
        generics = [$($generics:tt)*],
    ) => {
        impl <$($generics)*> $crate::table::Table for $row <$($generics)*>
        where
            Self: $crate::queryable::Row<ColumnName = &'static str>,
            $($generics: $crate::SpannerEncode,)*
            $($($pk_type)*: $crate::IntoSpanner + Clone,)*
        {
            const NAME: &'static str = $table_name;

            type Pk = $pk_name<$($($pk_type)*,)*>;

            fn into_row(self) -> ::core::result::Result<$crate::Row, $crate::error::ConvertError> {
                Ok($crate::Row::from(vec![
                    $(
                        (($($encode_with)*)(self.$field))?.into_protobuf(),
                    )*
                ]))
            }
        }

        $crate::__impl_pk! {
            table = $row,
            pk_name = $pk_name,
            pks = [$(($pk_field, ($($pk_type)*), $pk_index, $pk_generic)),*],
            generics = [$($generics)*],
        }

        const _: () = {
            const PKS: &[(&str, usize)] = &[
                $((stringify!($pk_field), $pk_index)),*
            ];

            if PKS.is_empty() {
                panic!(concat!(stringify!($row), " table must define at least 1 primary key field"));
            }


            if PKS[0].1 != 1 {
                panic!(concat!(stringify!($row), " primary key indices must start with 1"));
            }

            let mut index = 0;
            while index < PKS.len() {
                if PKS[index].1 != index + 1 {
                    panic!(concat!(stringify!($row), " found unordered pk index"));
                }

                index += 1;
            }
        };
    };
    ($($t:tt)*) => {
        $crate::__invalid_row_syntax!("row_impls" $($t)*);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_row_builder {
    (
        row = {
            imports = [$($imports:tt)*],
            meta = [$($meta:tt)*],
            row = $row:ident,
            row_vis = $row_vis:vis,
            table_name = [$($table_name:expr)?],
        },
        columns = [$(
            {
                field = $field:ident,
                field_vis = $field_vis:vis,
                field_name = [$field_name:expr],
                ty = ($($column_ty:tt)*),
                meta = [$($column_metas:tt)*],
                encode_with = [$($encode_with:tt)*],
                decode_with = [$($decode_with:tt)*],
                column_index = $col_idx:expr,
                generic = $generic:tt,
            }
        ),+ $(,)?],
        pks = [$(($pk_field:ident,($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],
        pk_name = [$pk_name:ident],
        generics = [$($generics:tt)*],
    ) => {
        $crate::__macro_internals::paste! {
            pub struct [<Partial $row:camel>]<$([<$field:camel>] = (),)*> {
                $($field_vis $field: [<$field:camel>],)*
            }
        }


        $crate::__impl_row_builder_methods! {
            prev_columns = [],
            curr_column = [],
            next_columns = [
                $(
                    {
                        field = $field,
                        field_vis = $field_vis,
                        field_name = [$field_name],
                        ty = ($($column_ty)*),
                        meta = [$($column_metas)*],
                        encode_with = [$($encode_with)*],
                        decode_with = [$($decode_with)*],
                        column_index = $col_idx,
                        generic = $generic,
                    }
                ),+
            ],
            row = {
                imports = [$($imports)*],
                meta = [$($meta)*],
                row = $row,
                row_vis = $row_vis,
                table_name = [$($table_name)?],
            },
            pks = [$(($pk_field, ($($pk_type)*), $pk_index, $pk_generic)),*],
            pk_name = [$pk_name],
            generics = [$($generics)*],
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_row_builder_methods {
    // first column, non-generic
    (
        prev_columns =
        [],curr_column =
        [
            {
                field =
                $curr_field:ident,field_vis =
                $curr_field_vis:vis,field_name =
                [$curr_field_name:expr],ty =
                ($($curr_column_ty:tt)*),meta =
                [$($curr_column_metas:tt)*],encode_with =
                [$($curr_encode_with:tt)*],decode_with =
                [$($curr_decode_with:tt)*],column_index =
                $curr_col_idx:expr,generic = no,
            }
        ],next_columns =
        [
            $(
                {
                    field =
                    $field:ident,field_vis =
                    $field_vis:vis,field_name =
                    [$field_name:expr],ty =
                    ($($column_ty:tt)*),meta =
                    [$($column_metas:tt)*],encode_with =
                    [$($encode_with:tt)*],decode_with =
                    [$($decode_with:tt)*],column_index =
                    $col_idx:expr,generic = $generic:tt,
                }
            ),* $(,)?
        ],row =
        {
            imports =
            [$($imports:tt)*],meta =
            [$($meta:tt)*],row =
            $row:ident,row_vis =
            $row_vis:vis,table_name =
            [$($table_name:expr)?],
        },pks =
        [$(($pk_field:ident,($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],pk_name =
        [$pk_name:ident],generics =
        [$($generics:tt)*],
    ) => {
        $crate::__macro_internals::paste! {
            impl [<Partial $row:camel>] {
                pub fn $curr_field(self, value: impl Into<$($curr_column_ty)*>) -> [<Partial $row:camel>]<$($curr_column_ty)*> {
                    [<Partial $row:camel>] {
                        $curr_field: value.into(),
                        $($field: (),)*
                    }
                }
            }

            impl [<Partial $row:camel>]<$($curr_column_ty)*> {
                pub fn $curr_field(value: impl Into<$($curr_column_ty)*>) -> [<Partial $row:camel>]<$($curr_column_ty)*> {
                    [<Partial $row:camel>] {
                        $curr_field: value.into(),
                        $($field: (),)*
                    }
                }
            }
        }
    };
    // first column, generic
    (
        prev_columns =
        [],curr_column =
        [
            {
                field =
                $curr_field:ident,field_vis =
                $curr_field_vis:vis,field_name =
                [$curr_field_name:expr],ty =
                ($($curr_column_ty:tt)*),meta =
                [$($curr_column_metas:tt)*],encode_with =
                [$($curr_encode_with:tt)*],decode_with =
                [$($curr_decode_with:tt)*],column_index =
                $curr_col_idx:expr,generic = yes,
            }
        ],next_columns =
        [
            $(
                {
                    field =
                    $field:ident,field_vis =
                    $field_vis:vis,field_name =
                    [$field_name:expr],ty =
                    ($($column_ty:tt)*),meta =
                    [$($column_metas:tt)*],encode_with =
                    [$($encode_with:tt)*],decode_with =
                    [$($decode_with:tt)*],column_index =
                    $col_idx:expr,generic =
                    $generic:tt,
                }
            ),* $(,)?
        ],row =
        {
            imports =
            [$($imports:tt)*],meta =
            [$($meta:tt)*],row =
            $row:ident,row_vis =
            $row_vis:vis,table_name =
            [$($table_name:expr)?],
        },pks =
        [$(($pk_field:ident,($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],pk_name =
        [$pk_name:ident],generics =
        [$($generics:tt)*],
    ) => {
        $crate::__macro_internals::paste! {
            impl [<Partial $row:camel>] {
                pub fn $curr_field<Value>(value: Value) -> [<Partial $row:camel>]<Value> {
                    [<Partial $row:camel>] {
                        $curr_field: value,
                        $($field: (),)*
                    }
                }
            }
        }
    };
    // not first & non-generic
    (
        prev_columns =
        [$(
            {
                field =
                $prev_field:ident,field_vis =
                $prev_field_vis:vis,field_name =
                [$prev_field_name:expr],ty =
                ($($prev_column_ty:tt)*),meta =
                [$($prev_column_metas:tt)*],encode_with =
                [$($prev_encode_with:tt)*],decode_with =
                [$($prev_decode_with:tt)*],column_index =
                $prev_col_idx:expr,
                generic = $prev_generic:tt,
            }
        ),*],curr_column =
        [
            {
                field =
                $curr_field:ident,field_vis =
                $curr_field_vis:vis,field_name =
                [$curr_field_name:expr],ty =
                ($($curr_column_ty:tt)*),meta =
                [$($curr_column_metas:tt)*],encode_with =
                [$($curr_encode_with:tt)*],decode_with =
                [$($curr_decode_with:tt)*],column_index =
                $curr_col_idx:expr,generic = no,
            }
        ],next_columns =
        [
            $(
                {
                    field =
                    $field:ident,field_vis =
                    $field_vis:vis,field_name =
                    [$field_name:expr],ty =
                    ($($column_ty:tt)*),meta =
                    [$($column_metas:tt)*],encode_with =
                    [$($encode_with:tt)*],decode_with =
                    [$($decode_with:tt)*],column_index =
                    $col_idx:expr,generic = $generic:tt,
                }
            ),* $(,)?
        ],row =
        {
            imports =
            [$($imports:tt)*],meta =
            [$($meta:tt)*],row =
            $row:ident,row_vis =
            $row_vis:vis,table_name =
            [$($table_name:expr)?],
        },pks =
        [$(($pk_field:ident,($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],pk_name =
        [$pk_name:ident],generics =
        [$($generics:tt)*],
    ) => {
        $crate::__macro_internals::paste! {
            impl [<Partial $row:camel>] {
                pub fn $curr_field(self, value: impl Into<$($curr_column_ty)*>) -> [<Partial $row:camel>]<$($curr_column_ty)*> {
                    [<Partial $row:camel>] {
                        $curr_field: value.into(),
                        $($field: (),)*
                    }
                }
            }

            impl [<Partial $row:camel>]<$($column_ty)*> {
                pub fn $curr_field(value: impl Into<$($curr_column_ty)*>) -> [<Partial $row:camel>]<$($curr_column_ty)*> {
                    [<Partial $row:camel>] {
                        $curr_field: value.into(),
                        $($field: (),)*
                    }
                }
            }
        }

        $crate::__impl_row_builder_methods! {
            prev_columns = [
                $({
                    field =
                    $prev_field,field_vis =
                    $prev_field_vis,field_name =
                    [$prev_field_name],ty =
                    ($($prev_column_ty)*),meta =
                    [$($prev_column_metas)*],encode_with =
                    [$($prev_encode_with)*],decode_with =
                    [$($prev_decode_with)*],column_index =
                    $prev_col_idx,
                    generic = $prev_generic,
                },)*
                {
                    field = $curr_field,
                    field_vis = $curr_field_vis,
                    field_name = [$curr_field_name],
                    ty = ($($curr_column_ty)*),
                    meta = [$($curr_column_metas)*],
                    encode_with = [$($curr_encode_with)*],
                    decode_with = [$($curr_decode_with)*],
                    column_index = $curr_col_idx,
                    generic = $curr_generic,
                },
            ],
            curr_column = [],
            next_columns = [
                $(
                    {
                        field =
                        $field,field_vis =
                        $field_vis,field_name =
                        [$field_name],ty =
                        ($($column_ty)*),meta =
                        [$($column_metas)*],encode_with =
                        [$($encode_with)*],decode_with =
                        [$($decode_with)*],column_index =
                        $col_idx,generic =
                        $generic,
                    },
                )*
            ],
            row =
            {
                imports =
                [$($imports)*],meta =
                [$($meta)*],row =
                $row,row_vis =
                $row_vis,table_name =
                    [$($table_name)?],
            },pks =
            [$(($pk_field,($($pk_type)*), $pk_index, $pk_generic)),* $(,)?],pk_name =
            [$pk_name],generics =
            [$($generics)*],
        }
    };
    // not first & generic
    (
        prev_columns =
        [$(
            {
                field =
                $prev_field:ident,field_vis =
                $prev_field_vis:vis,field_name =
                [$prev_field_name:expr],ty =
                ($($prev_column_ty:tt)*),meta =
                [$($prev_column_metas:tt)*],encode_with =
                [$($prev_encode_with:tt)*],decode_with =
                [$($prev_decode_with:tt)*],column_index =
                $prev_col_idx:expr,
                generic = $prev_generic:tt,
            }
        ),*],curr_column =
        [
            {
                field =
                $curr_field:ident,field_vis =
                $curr_field_vis:vis,field_name =
                [$curr_field_name:expr],ty =
                ($($curr_column_ty:tt)*),meta =
                [$($curr_column_metas:tt)*],encode_with =
                [$($curr_encode_with:tt)*],decode_with =
                [$($curr_decode_with:tt)*],column_index =
                $curr_col_idx:expr,generic = yes,
            }
        ],next_columns =
        [
            $(
                {
                    field =
                    $field:ident,field_vis =
                    $field_vis:vis,field_name =
                    [$field_name:expr],ty =
                    ($($column_ty:tt)*),meta =
                    [$($column_metas:tt)*],encode_with =
                    [$($encode_with:tt)*],decode_with =
                    [$($decode_with:tt)*],column_index =
                    $col_idx:expr,generic = $generic:tt,
                }
            ),* $(,)?
        ],row =
        {
            imports =
            [$($imports:tt)*],meta =
            [$($meta:tt)*],row =
            $row:ident,row_vis =
            $row_vis:vis,table_name =
            [$($table_name:expr)?],
        },pks =
        [$(($pk_field:ident,($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],pk_name =
        [$pk_name:ident],generics =
        [$($generics:tt)*],
    ) => {
        $crate::__macro_internals::paste! {
            impl [<Partial $row:camel>] {
                pub fn $curr_field<Value>(value: Value) -> [<Partial $row:camel>]<Value> {
                    [<Partial $row:camel>] {
                        $curr_field: value,
                        $($field: (),)*
                    }
                }
            }
        }

        $crate::__impl_row_builder_methods! {
            prev_columns = [
                $({
                    field =
                    $prev_field,field_vis =
                    $prev_field_vis,field_name =
                    [$prev_field_name],ty =
                    ($($prev_column_ty)*),meta =
                    [$($prev_column_metas)*],encode_with =
                    [$($prev_encode_with)*],decode_with =
                    [$($prev_decode_with)*],column_index =
                    $prev_col_idx,
                    generic = $prev_generic,
                },)*
                {
                    field = $curr_field,
                    field_vis = $curr_field_vis,
                    field_name = [$curr_field_name],
                    ty = ($($curr_column_ty)*),
                    meta = [$($curr_column_metas)*],
                    encode_with = [$($curr_encode_with)*],
                    decode_with = [$($curr_decode_with)*],
                    column_index = $curr_col_idx,
                    generic = $curr_generic,
                },
            ],
            curr_column = [],
            next_columns = [
                $(
                    {
                        field =
                        $field,field_vis =
                        $field_vis,field_name =
                        [$field_name],ty =
                        ($($column_ty)*),meta =
                        [$($column_metas)*],encode_with =
                        [$($encode_with)*],decode_with =
                        [$($decode_with)*],column_index =
                        $col_idx,generic =
                        $generic,
                    },
                )*
            ],
            row =
            {
                imports =
                [$($imports)*],meta =
                [$($meta)*],row =
                $row,row_vis =
                $row_vis,table_name =
                    [$($table_name)?],
            },pks =
            [$(($pk_field,($($pk_type)*), $pk_index, $pk_generic)),* $(,)?],pk_name =
            [$pk_name],generics =
            [$($generics)*],
        }
    };
    (
        prev_columns = $prev:tt,
        curr_column =
        [],
        next_columns =
        [
            {
                field =
                $curr_field:ident,field_vis =
                $curr_field_vis:vis,field_name =
                [$curr_field_name:expr],ty =
                ($($curr_column_ty:tt)*),meta =
                [$($curr_column_metas:tt)*],encode_with =
                [$($curr_encode_with:tt)*],decode_with =
                [$($curr_decode_with:tt)*],column_index =
                $curr_col_idx:expr,generic =
                $curr_generic:tt,
            }
            $(,)?
            $(
                {
                    field =
                    $field:ident,field_vis =
                    $field_vis:vis,field_name =
                    [$field_name:expr],ty =
                    ($($column_ty:tt)*),meta =
                    [$($column_metas:tt)*],encode_with =
                    [$($encode_with:tt)*],decode_with =
                    [$($decode_with:tt)*],column_index =
                    $col_idx:expr,generic =
                    $generic:tt,
                }
            ),* $(,)?
        ],
        $($rest:tt)*
    ) => {
        $crate::__impl_row_builder_methods! {
            prev_columns = $prev,
            curr_column = [{
                field = $curr_field,
                field_vis = $curr_field_vis,
                field_name = [$curr_field_name],
                ty = ($($curr_column_ty)*),
                meta = [$($curr_column_metas)*],
                encode_with = [$($curr_encode_with)*],
                decode_with = [$($curr_decode_with)*],
                column_index = $curr_col_idx,
                generic = $curr_generic,
            }],
            next_columns = [
                $(
                    {
                        field =
                        $field,field_vis =
                        $field_vis,field_name =
                        [$field_name],ty =
                        ($($column_ty)*),meta =
                        [$($column_metas)*],encode_with =
                        [$($encode_with)*],decode_with =
                        [$($decode_with)*],column_index =
                        $col_idx,generic =
                        $generic,
                    },
                )*
            ],
            $($rest)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_pk {
    (
        table = $table:ident,
        pk_name = $pk_name:ident,
        pks = [$(($pk_field:ident, ($($pk_type:tt)*), $pk_index:literal, $pk_generic:tt)),* $(,)?],
        generics = [$($generics:tt)*],
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        #[allow(non_camel_case_types)]
        pub struct $pk_name<$($pk_field = (),)*> {
            $(
                pub $pk_field: $pk_field,
            )*
        }

        impl<$($generics)*> $crate::pk::PrimaryKey for $pk_name<$($($pk_type)*,)*>
        where
            $table<$($generics)*>: $crate::Table<Pk = Self>,
            $($generics: $crate::IntoSpanner + Clone,)*
        {
            type Parts =  ($($($pk_type)*,)*);
            type Table = $table<$($generics)*>;

            #[inline]
            fn from_parts(parts: Self::Parts) -> Self {
                let ($($pk_field,)*) = parts;
                Self { $($pk_field,)* }
            }

            #[inline]
            fn into_parts(self) -> Self::Parts {
                (
                    $(self.$pk_field,)*
                )
            }
        }

        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk_name,
            first = yes,
            generics = [$($generics)*],
            prev_fields = [],
            curr_field = [],
            rest_fields = [$(($pk_field, ($($pk_type)*), $pk_generic),)*],
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_pk_builder_fns {
    // current field is not the first pk field, and isnt generic
    (
        table = $table:ident,
        pk = $pk:ident,
        first = no,
        generics = [$($generics:tt)*],
        prev_fields = [$(($prev_pk_field:ident, ($($prev_pk_type:tt)*), $prev_pk_generic:tt),)*],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*), no],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:tt)*), $rest_pk_generic:tt),)*],
    ) => {
        $crate::__macro_internals::paste! {

            impl<$($generics)*> $pk<$($($prev_pk_type)*,)* ()> {
                #[inline]
                pub fn $curr_pk_field<I>(self, $curr_pk_field: I) -> $pk<$($($prev_pk_type)*,)* $($curr_pk_type)*>
                where
                    I: Into<$($curr_pk_type)*>,
                {
                    $pk {
                        $($prev_pk_field: self.$prev_pk_field,)*
                        $curr_pk_field: <I as ::core::convert::Into<$($curr_pk_type)*>>::into($curr_pk_field),
                        $($rest_pk_field: (),)*
                    }
                }

                #[inline]
                pub fn [<$curr_pk_field:snake _range>]<__KeyPart>(
                    self,
                    bounds: impl ::core::ops::IntoBounds<__KeyPart>,
                ) -> impl $crate::key_set::IntoKeyRange<$table<$($generics)*>>
                where
                    __KeyPart: $crate::IntoSpanner<SpannerType = <$($curr_pk_type)* as $crate::IntoSpanner>::SpannerType>,
                    $($($prev_pk_type)*: $crate::IntoSpanner + Clone,)*
                    $($curr_pk_type)*: $crate::IntoSpanner + Clone,
                {
                    $crate::key_set::make_range_for_final_component(self, bounds)
                }
            }
        }

        impl<$($generics)*> $crate::pk::PartialPkParts<$table<$($generics)*>> for ($($($prev_pk_type)*,)* $($curr_pk_type)*,)
        where
            $($($prev_pk_type)*: $crate::IntoSpanner + Clone,)*
            $($curr_pk_type)*: $crate::IntoSpanner + Clone,
        { }

        impl<$($generics)*> $crate::pk::IntoPartialPkParts<$table<$($generics)*>> for $pk<$($($prev_pk_type)*,)* $($curr_pk_type)*>
        where
            $($($prev_pk_type)*: $crate::IntoSpanner + Clone,)*
            $($curr_pk_type)*: $crate::IntoSpanner + Clone,
        {
            type PartialParts = ($($($prev_pk_type)*,)* $($curr_pk_type)*,);

            fn into_partial_parts(self) -> Self::PartialParts {
                (
                    $(self.$prev_pk_field,)*
                    self.$curr_pk_field,
                )
            }
        }


        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = no,
            generics = [$($generics)*],
            prev_fields = [
                $(($prev_pk_field, ($($prev_pk_type)*), $prev_pk_generic),)*
                ($curr_pk_field, ($($curr_pk_type)*), no),
            ],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*), $rest_pk_generic),)*],
        }
    };
    (
        table = $table:ident,
        pk = $pk:ident,
        first = no,
        generics = [$($generics:tt)*],
        prev_fields = [$(($prev_pk_field:ident, ($($prev_pk_type:tt)*), $prev_pk_generic:tt),)*],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*), yes],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:tt)*), $rest_pk_generic:tt))*],
    ) => {
        $crate::__macro__internals::paste! {
            impl<$($generics)*> $pk<$($($prev_pk_type)*,)* ()> {
                #[inline]
                pub fn $curr_pk_field<$($curr_pk_type)*>(self, $curr_pk_field: $($curr_pk_type)*) -> $pk<$($($prev_pk_type)*,)* $($curr_pk_type)*>
                {
                    $pk {
                        $($prev_pk_field: self.$prev_pk_field,)*
                        $curr_pk_field,
                        $($rest_pk_field: (),)*
                    }
                }

                #[inline]
                pub fn [<$curr_pk_field:snake _range>](
                    self,
                    bounds: impl ::core::ops::IntoBounds<$($curr_pk_type)*>,
                ) -> impl $crate::key_set::IntoKeyRange<$table<$($generics)*>>
                where
                    $($curr_pk_type)*: $crate::IntoSpanner + Clone,
                {
                    $crate::key_set::make_range_for_final_component(self, bounds)
                }
            }
        }

        impl<$($generics)*> $crate::pk::PartialPkParts<$table<$($generics)*>> for ($($($prev_pk_type)*,)* $($curr_pk_type)*,)
        where
            $($($prev_pk_type)*: $crate::IntoSpanner + Clone,)*
            $($curr_pk_type)*: $crate::IntoSpanner + Clone,
        { }

        impl<$($generics)*> $crate::pk::IntoPartialPkParts<$table<$($generics)*>> for $pk<$($($prev_pk_type)*,)* $($curr_pk_type)*>
        where
            $($($prev_pk_type)*: $crate::IntoSpanner + Clone,)*
            $($curr_pk_type)*: $crate::IntoSpanner + Clone,
        {
            type PartialParts = ($($($prev_pk_type)*,)* $($curr_pk_type)*,);

            fn into_partial_parts(self) -> Self::PartialParts {
                (
                    $(self.$prev_pk_field,)*
                    self.$curr_pk_field,
                )
            }

        }


        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = no,
            generics = [$($generics)*],
            prev_fields = [
                $(($prev_pk_field, ($($prev_pk_type)*), $prev_pk_generic),)*
                ($curr_pk_field, ($($curr_pk_type)*), yes),
            ],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*), $rest_pk_generic),)*],
        }
    };
    // handle the first field, only difference from the above block is we make the first pk builder function
    // not require 'self', since that would then require us to do a Pk::default().first_key(...)
    (
        table = $table:ident,
        pk = $pk:ident,
        first = yes,
        generics = [$($generics:tt)*],
        prev_fields = [],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*), no],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:ty)*), $rest_pk_generic:tt),)*],
    ) => {
        impl $pk<()> {
            #[inline]
            pub fn $curr_pk_field<I>($curr_pk_field: I) -> $pk<$($curr_pk_type)*>
            where
                I: Into<$($curr_pk_type)*>,
            {
                $pk {
                    $curr_pk_field: <I as ::core::convert::Into<$($curr_pk_type)*>>::into($curr_pk_field),
                    $($rest_pk_field: (),)*
                }
            }

        }

        impl<$($generics)*> $crate::pk::PartialPkParts<$table<$($generics)*>> for ($($curr_pk_type)*,) { }

        impl<$($generics)*> $crate::pk::IntoPartialPkParts<$table<$($generics)*>> for $pk<$($curr_pk_type)*> {
            type PartialParts = ($($curr_pk_type)*,);

            fn into_partial_parts(self) -> Self::PartialParts {
                (
                    self.$curr_pk_field,
                )
            }
        }


        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = no,
            generics = [$($generics)*],
            prev_fields = [($curr_pk_field, ($($curr_pk_type)*), no),],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*), $rest_pk_generic),)*],
        }
    };
    // same as above, but with a generic type.
    (
        table = $table:ident,
        pk = $pk:ident,
        first = yes,
        generics = [$($generics:tt)*],
        prev_fields = [],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*), yes],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:ty)*), $rest_pk_generic:tt),)*],
    ) => {
        $crate::__macro_internals::paste! {
            impl $pk<()> {
                #[inline]
                pub fn $curr_pk_field<$($curr_pk_type)*>($curr_pk_field: $($curr_pk_type)*) -> $pk<$($curr_pk_type)*> {
                    $pk {
                        $curr_pk_field,
                        $($rest_pk_field: (),)*
                    }
                }

                #[inline]
                pub fn [<$curr_pk_field:snake _range>]<$($curr_pk_type)*>(
                    bounds: impl ::core::ops::IntoBounds<$($curr_pk_type)*>,
                ) -> impl $crate::key_set::IntoKeyRange<$table<$($generics)*>>
                where
                    $($curr_pk_type)*: $crate::IntoSpanner + Clone,
                {
                    let (start, end) = ::core::ops::IntoBounds::into_bounds(bounds);
                    $crate::key_set::convert_to_range(start.map(Self::$curr_pk_field), end.map(Self::$curr_pk_field))
                }
            }
        }

        impl<$($generics)*> $crate::pk::PartialPkParts<$table<$($generics)*>> for ($($curr_pk_type)*,)
        where
            $table<$($generics)*>: $crate::Table,
            $($generics: $crate::IntoSpanner,)*
        { }

        impl<$($generics)*> $crate::pk::IntoPartialPkParts<$table<$($generics)*>> for $pk<$($curr_pk_type)*>
        where
            $table<$($generics)*>: $crate::Table,
            $($generics: $crate::IntoSpanner,)*
        {
            type PartialParts = ($($curr_pk_type)*,);

            fn into_partial_parts(self) -> Self::PartialParts {
                (
                    self.$curr_pk_field,
                )
            }
        }


        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = no,
            generics = [$($generics)*],
            prev_fields = [($curr_pk_field, ($($curr_pk_type)*), yes),],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*), $rest_pk_generic),)*],
        }
    };
    // grab a new 'current'
    (
        table = $table:ident,
        pk = $pk:ident,
        first = $first:tt,
        generics = [$($generics:tt)*],
        prev_fields = $prev_fields:tt,
        curr_field = [],
        rest_fields = [
            ($next_field:ident, ($($next_type:tt)*), $next_pk_generic:tt),
            $(($rest_pk_field:ident, ($($rest_pk_type:ty)*), $rest_pk_generic:tt),)*
        ],
    ) => {
        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = $first,
            generics = [$($generics)*],
            prev_fields = $prev_fields,
            curr_field = [$next_field, ($($next_type)*), $next_pk_generic],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*), $rest_pk_generic),)*],
        }
    };

    // done
    (
        table = $table:ident,
        pk = $pk:ident,
        first = $ignore:tt,
        generics = [$($generics:tt)*],
        prev_fields = [$($prev:tt)*],
        curr_field = [],
        rest_fields = [],
    ) => {
    };
    ($($t:tt)*) => {
        $crate::__invalid_row_syntax!("__impl_pk_builder_fs" $($t)*);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __replace_with_unit {
    ($e:expr) => {
        ()
    };
}
