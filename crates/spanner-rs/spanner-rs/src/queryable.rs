use generic_array::{ArrayLength, GenericArray};

use crate::FromSpanner;
use crate::column::{Column, Unnamed};
use crate::results::RawRow;

pub trait Queryable: Sized {
    type NumColumns: ArrayLength;
    type ColumnName;

    const COLUMNS: GenericArray<Column<'static, Self::ColumnName>, Self::NumColumns>;

    fn from_row(row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self>;
}

// helper impl for decoding a row made up of a single column
impl<T: FromSpanner> Queryable for T {
    type ColumnName = Unnamed;
    type NumColumns = typenum::U1;

    const COLUMNS: GenericArray<Column<'static, Self::ColumnName>, Self::NumColumns> =
        GenericArray::from_array([Column::unnamed::<T>(0)]);

    #[inline]
    fn from_row(mut row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self> {
        row.decode_at_index(0, T::from_field_and_value)
    }
}

macro_rules! count_idents {
    ($next:ident, $($i:ident),+ $(,)?) => {
        1 + count_idents!($($i,)+)
    };
    ($last:ident $(,)?) => { 1 }
}

macro_rules! impl_queryable_for_tuples {
    (
        $($t:ident: $index:literal),* $(,)?
    ) => {
        impl<$($t,)*> Queryable for ($($t,)*)
        where
            $($t: FromSpanner,)*
        {
            type NumColumns = typenum::U<{ count_idents!($($t,)*) }>;
            type ColumnName = Unnamed;

            const COLUMNS: GenericArray<Column<'static, Unnamed>, Self::NumColumns> = GenericArray::from_array([
                $(
                    Column::unnamed::<$t>($index),
                )*
            ]);

            fn from_row(mut row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self> {
                Ok((
                    $(
                        row.decode_at_index($index, <$t>::from_field_and_value)?,
                    )*
                ))
            }
        }
    };
}

impl_queryable_for_tuples!(A: 0);
impl_queryable_for_tuples!(A: 0, B: 1);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
impl_queryable_for_tuples!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);

pub mod new {
    // TODO: figure out how to make tuple indexing at a type level work
    // so we can replace Queryable::COLUMNS with a type parameter
    // `type Columns: Columns;` instead
    use typenum::{IsLess, True, U, Unsigned};

    use crate::ty::SpannerType;

    pub trait Columns {
        type Number: Unsigned;

        type ValueAt<Index: Unsigned + IsLess<Self::Number, Output = True>>: Column;
    }

    pub trait Column {
        const NAME: &'static str;
        type Type: SpannerType;
        type Index: Unsigned;
    }

    pub trait ColumnAt<Index: Unsigned + IsLess<Self::Number, Output = True>>: Columns {
        type Value: Column;
    }

    impl<Idx, T: Column> ColumnAt<Idx> for (T,)
    where
        Idx: Unsigned + IsLess<Self::Number, Output = True>,
    {
        type Value = T;
    }

    impl<T> Columns for (T,)
    where
        T: Column,
    {
        type Number = U<1>;
        type ValueAt<Index: Unsigned + IsLess<Self::Number, Output = True>> =
            <Self as ColumnAt<Index>>::Value;
    }
}
