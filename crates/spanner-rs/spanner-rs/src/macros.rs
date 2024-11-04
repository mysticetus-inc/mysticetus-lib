/// Defines a struct as a spanner table, and implements all of the nessecary traits.
///
/// Should be used within its own module, as names will clash if 2 invocations of
/// [`table!`] are called in the same module, unless certain options are used (to override the
/// defaults)
///
/// Notes + Limitations:
///     - functions/modules specified in `encode_with`/`decode_with`/`with` field options need to be
///       single identifiers (i.e can't be a path like `crate::util::...`).
#[macro_export]
macro_rules! table {
    // adapted (heavily) from the diesel::table macro
    ($($tokens:tt)*) => {
        $crate::__parse_table! {
            tokens = [$($tokens)*],
            imports = [],
            meta = [],
            unprocessed_spanner_meta = [],
            table = unknown,
            table_vis = unknown,
            table_name = unknown,
            pk_name = [PrimaryKey],
            pks = [],
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __invalid_table_syntax {
    () => {
        compile_error!("Invalid `table!` syntax");
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_table {
    // Found an import
    (
        tokens = [use $($import:tt)::+; $($rest:tt)*],
        imports = [$($imports:tt)*],
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = [$($rest)*],
            imports = [$($imports)* use $($import)::+;],
            $($args)*
        }
    };
    // we found a spanner table attribute
    (
        tokens = [#[spanner($($attrs:tt)*)] $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$($prev:tt)*],
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($prev:tt)* $($attrs)*],
            $($args)*
        }
    };

    // Found table_name attribute, override whatever we had before
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? table_name = $table_name:literal $($rest_unprocessed:tt)*],
        table = $table:tt,
        table_vis = $table_vis:tt,
        table_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = $tokens,
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($rest_unprocessed)*],
            table = $table,
            table_vis = $table_vis,
            table_name = [$table_name],
            $($args)*
        }
    };

    // Found pk_name attribute, override whatever we had before
    (
        tokens = $tokens:tt,
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)? pk_name = $pk_name:ident $($rest_unprocessed:tt)*],
        table = $table:tt,
        table_vis = $table_vis:tt,
        table_name = $table_name:tt,
        pk_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = $tokens,
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [$($rest_unprocessed)*],
            table = $table,
            table_vis = $table_vis,
            table_name = $table_name,
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
        $crate::__parse_table! {
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
        compile_error!(concat!("unknown spanner table option `", stringify!($unknown), "`"));
    };
    // Found the table/struct definition
    (
        tokens = [$table_vis:vis struct $table:ident $($rest:tt)* ],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [$(,)?],
        table = $ignore:tt,
        table_vis = $ignore2:tt,
        table_name = $table_name:tt,
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [],
            table = $table,
            table_vis = $table_vis,
            table_name = $table_name,
            $($args)*
        }
    };

    // Reached columns with no table_name, set a default
    (
        tokens = [{$($columns:tt)*}],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [],
        table = $table:tt,
        table_vis = $table_vis:tt,
        table_name = unknown,
        $($args:tt)*
    ) => {
        $crate::__parse_table! {
            tokens = [{$($columns)*}],
            imports = $imports,
            meta = $meta,
            unprocessed_spanner_meta = [],
            table = $table,
            table_vis = $table_vis,
            table_name = [stringify!($table)],
            $($args)*
        }
    };

    // Parse the columns
    (
        tokens = [{$($columns:tt)*}],
        imports = $imports:tt,
        meta = $meta:tt,
        unprocessed_spanner_meta = [],
        table = $table:tt,
        table_vis = $table_vis:tt,
        table_name = $table_name:tt,
        pk_name = $pk_name:tt,
        pks = [],
    ) => {
        $crate::__parse_columns! {
            tokens = [$($columns)*],
            next_column_index = [0],
            table = {
                imports = $imports,
                meta = $meta,
                table = $table,
                table_vis = $table_vis,
                table_name = $table_name,
            },
            columns = [],
            pks = [],
            pk_name = $pk_name,
        }
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        compile_error!(concat!($(stringify!($tokens)),*));
        $crate::__invalid_table_syntax!();
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
                field_name = [$crate::__macro_internals::static_casing::pascal_case!(ident -> lit; $field)],
                ty = ($($ty)::* $(<$($ty_params)::*>)*),
                meta = [],
                encode_with = unknown,
                decode_with = unknown,
                column_index = $next_col_idx,
                pk_index = unknown,
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
                field_name = [$crate::__macro_internals::static_casing::pascal_case!(ident -> lit; $field)],
                ty = ($ty),
                meta = [],
                encode_with = unknown,
                decode_with = unknown,
                column_index = $next_col_idx,
                pk_index = unknown,
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
            field_name = [$field_name:expr],
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
                field_name = [$field_name],
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
            field_name = [$field_name:expr],
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
                field_name = [$field_name],
                ty = $ty,
                meta = [$($meta)* #$new_meta],
                $($current_column)*
            },
            $($args)*
        }
    };

    // parsing spanner field rename
    (
        current_column = {
            unchecked_meta = [],
            spanner_args = [$(,)? rename = $rename_as:literal $($rest_spanner_args:tt)*],
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
        },
        tokens = $tokens:tt,
        next_column_index = $next_col_idx:tt,
        table = $table:tt,
        columns = [$($columns:tt,)*],
        $($args:tt)*
    ) => {
        $crate::__parse_columns! {
            tokens = $tokens,
            next_column_index = $next_col_idx,
            table = $table,
            columns = [$($columns,)* {
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
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
        },
        tokens = $tokens:tt,
        next_column_index = $next_col_idx:tt,
        table = $table:tt,
        columns = [$($columns:tt,)*],
        pks = [$($existing_pks:tt)*],
        pk_name = $pk_name:tt,
    ) => {
        $crate::__parse_columns! {
            tokens = $tokens,
            next_column_index = $next_col_idx,
            table = $table,
            columns = [$($columns,)* {
                field = $field,
                field_vis = $field_vis,
                field_name = $field_name,
                ty = $ty,
                meta = $meta,
                encode_with = $encode_with,
                decode_with = $decode_with,
                column_index = $col_idx,
            },],
            pks = [$($existing_pks)* ($field, $ty, $pk_index),],
            pk_name = $pk_name,
        }
    };

    // Done parsing all columns
    (
        tokens = [],
        next_column_index = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__table_impls!($($args)*);
    };

    ($($tokens:tt)*) => {
        compile_error!(concat!($(stringify!($tokens)),*));
        $crate::__invalid_table_syntax!();
    }
}

/*
// TODO: no real need for column implmenentations, we arent doing crazy queries yet.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_col {
    (
        field = $field:ident,
        field_ident = [$($field_ident:tt)*],
        field_ty = ($($field_ty:tt)*),
        col_index = $col_index:expr,
    ) => {
        $crate::__impl_col! {
            @INTERNAL
            field = $field,
            field_ident = $($field_ident)*,
            field_ty = ($($field_ty)*),
            col_index = $col_index,
        }
    };
    (
        @INTERNAL
        field = $field:ident,
        field_ident = $field_ident:ident,
        field_ty = ($($field_ty:tt)*),
        col_index = $col_index:expr,
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $field_ident($($field_ty)*);
    };
}
*/

#[macro_export]
#[doc(hidden)]
macro_rules! __table_impls {
    (
        table = {
            imports = [$($imports:tt)*],
            meta = [$($meta:tt)*],
            table = $table:ident,
            table_vis = $table_vis:vis,
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
                }
            ),+
            $(,)?
        ],
        pks = [$(($pk_field:ident, ($($pk_type:tt)*), $pk_index:literal)),* $(,)?],
        pk_name = [$pk_name:ident],
    ) => {
        $($meta)*
        $table_vis struct $table {
            $(
                $($column_metas)*
                $field_vis $field: $($column_ty)*,
            )+
        }


        impl $crate::queryable::Queryable for $table {
            type NumColumns = $crate::__macro_internals::typenum::U<{ <[()]>::len(&[$($crate::__replace_with_unit!($field),)*]) }>;

            const COLUMNS: $crate::__macro_internals::generic_array::GenericArray<$crate::column::Column<'static>, Self::NumColumns> = $crate::__macro_internals::generic_array::GenericArray::from_array([
                $(
                    $crate::column::Column::new::<$($column_ty)*>($col_idx, $field_name),
                )*
            ]);

            fn from_row(mut row: $crate::results::RawRow<'_, Self::NumColumns>) -> $crate::Result<Self> {
                Ok(Self {
                    $(
                        $field: row.decode_at_index($col_idx, $($decode_with)*)?,
                    )*
                })
            }
        }

        impl $crate::insertable::Insertable for $table {
            fn into_row(self) -> ::core::result::Result<$crate::Row, $crate::error::ConvertError> {
                Ok($crate::Row::from(vec![
                    $(
                        (($($encode_with)*)(self.$field))?.into_protobuf(),
                    )*
                ]))
            }
        }

        impl $crate::table::Table for $table {
            const NAME: &'static str = $table_name;

            type Pk = $pk_name<$($($pk_type)*,)*>;
        }

        $crate::__impl_pk! {
            table = $table,
            pk_name = $pk_name,
            pks = [$(($pk_field, ($($pk_type)*), $pk_index)),*],
        }

        const _: () = {
            const PKS: &[(&str, usize)] = &[
                $((stringify!($pk_field), $pk_index)),*
            ];

            if PKS.is_empty() {
                panic!(concat!(stringify!($table), " table must define at least 1 primary key field"));
            }


            if PKS[0].1 != 1 {
                panic!(concat!(stringify!($table), " primary key indices must start with 1"));
            }

            let mut index = 0;
            while index < PKS.len() {
                if PKS[index].1 != index + 1 {
                    panic!(concat!(stringify!($table), " found unordered pk index"));
                }

                index += 1;
            }
        };
    };
    ($($t:tt)*) => {
        compile_error!(concat!(
            $(stringify!($t)),*
        ));
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_pk {
    (
        table = $table:ident,
        pk_name = $pk_name:ident,
        pks = [$(($pk_field:ident, ($($pk_type:ty)*), $pk_index:literal)),* $(,)?],
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        #[allow(non_camel_case_types)]
        pub struct $pk_name<$($pk_field = (),)*> {
            $(
                pub $pk_field: $pk_field,
            )*
        }

        impl $crate::pk::PrimaryKey for $pk_name<$($($pk_type)*,)*> {
            type Parts =  ($($($pk_type)*,)*);
            type Table = $table;

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
            prev_fields = [],
            curr_field = [],
            rest_fields = [$(($pk_field, ($($pk_type)*)),)*],
        }

    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_pk_builder_fns {
    (
        table = $table:ident,
        pk = $pk:ident,
        first = no,
        prev_fields = [$(($prev_pk_field:ident, ($($prev_pk_type:tt)*)),)*],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*)],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:tt)*)),)*],
    ) => {
        impl $pk<$($($prev_pk_type)*,)* ()> {
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

        }

        impl $crate::pk::PartialPkParts<$table> for ($($($prev_pk_type)*,)* $($curr_pk_type)*,) { }

        impl $crate::pk::IntoPartialPkParts<$table> for $pk<$($($prev_pk_type)*,)* $($curr_pk_type)*> {
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
            prev_fields = [$(($prev_pk_field, ($($prev_pk_type)*)),)* ($curr_pk_field, ($($curr_pk_type)*)),],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*)),)*],
        }
    };
    // handle the first field, only difference from the above block is we make the first pk builder function
    // not require 'self', since that would then require us to do a Pk::default().first_key(...)
    (
        table = $table:ident,
        pk = $pk:ident,
        first = yes,
        prev_fields = [],
        curr_field = [$curr_pk_field:ident, ($($curr_pk_type:tt)*)],
        rest_fields = [$(($rest_pk_field:ident, ($($rest_pk_type:ty)*)),)*],
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

        impl $crate::pk::PartialPkParts<$table> for ($($curr_pk_type)*,) { }

        impl $crate::pk::IntoPartialPkParts<$table> for $pk<$($curr_pk_type)*> {
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
            prev_fields = [($curr_pk_field, ($($curr_pk_type)*)),],
            curr_field = [],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*)),)*],
        }
    };
    // grab a new 'current'
    (
        table = $table:ident,
        pk = $pk:ident,
        first = $first:tt,
        prev_fields = $prev_fields:tt,
        curr_field = [],
        rest_fields = [($next_field:ident, ($($next_type:tt)*)), $(($rest_pk_field:ident, ($($rest_pk_type:ty)*)),)*],
    ) => {
        $crate::__impl_pk_builder_fns! {
            table = $table,
            pk = $pk,
            first = $first,
            prev_fields = $prev_fields,
            curr_field = [$next_field, ($($next_type)*)],
            rest_fields = [$(($rest_pk_field, ($($rest_pk_type)*)),)*],
        }
    };

    // done
    (
        table = $table:ident,
        pk = $pk:ident,
        first = $ignore:tt,
        prev_fields = [$($prev:tt)*],
        curr_field = [],
        rest_fields = [],
    ) => {
    };
    ($($t:tt)*) => {
        compile_error!(concat!(
            $(stringify!($t),)*
        ))
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __replace_with_unit {
    ($e:expr) => {
        ()
    };
}
