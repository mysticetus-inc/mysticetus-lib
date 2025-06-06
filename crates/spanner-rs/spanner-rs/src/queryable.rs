use generic_array::{ArrayLength, GenericArray};

use crate::column::{Column, Unnamed};
use crate::results::RawRow;
use crate::{FromSpanner, SpannerEncode};

/// Base trait defining the columns of a row of spanner data.
pub trait Row {
    type NumColumns: ArrayLength;
    /// Should either be [&'static str] for a named column, or
    /// [Unnamed] for an unnamed one.
    type ColumnName;

    const COLUMNS: GenericArray<Column<'static, Self::ColumnName>, Self::NumColumns>;
}

/// Defines a row type that can be parsed from a raw protobuf row
pub trait Queryable: Row + Sized {
    fn from_row(row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self>;
}

// helper impls for decoding a row made up of a single column
impl<T: SpannerEncode + FromSpanner<SpannerType = <T as SpannerEncode>::SpannerType>> Row for T {
    type ColumnName = Unnamed;
    type NumColumns = typenum::U1;

    const COLUMNS: GenericArray<Column<'static, Self::ColumnName>, Self::NumColumns> =
        GenericArray::from_array([Column::unnamed::<<T as SpannerEncode>::SpannerType>(0)]);
}
impl<T: SpannerEncode + FromSpanner<SpannerType = <T as SpannerEncode>::SpannerType>> Queryable
    for T
{
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
        impl<$($t,)*> Row for ($($t,)*)
        where
            $($t: SpannerEncode,)*
        {
            type NumColumns = typenum::U<{ count_idents!($($t,)*) }>;
            type ColumnName = Unnamed;

            const COLUMNS: GenericArray<Column<'static, Unnamed>, Self::NumColumns> = GenericArray::from_array([
                $(
                    Column::unnamed::<<$t>::SpannerType>($index),
                )*
            ]);
        }


        impl<$($t,)*> Queryable for ($($t,)*)
        where
            $($t: FromSpanner + SpannerEncode,)*
        {
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
    use std::fmt;
    use std::marker::PhantomData;

    // TODO: figure out how to make tuple indexing at a type level work
    // so we can replace Queryable::COLUMNS with a type parameter
    // `type Columns: Columns;` instead
    use typenum::{IsLess, True, U, Unsigned};

    use crate::SpannerEncode;
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

    pub struct DebugColumn<C>(PhantomData<C>);

    impl<C: Column> fmt::Debug for DebugColumn<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Column")
                .field("name", &C::NAME)
                .field("index", &<C::Index as Unsigned>::USIZE)
                .field("type", &crate::ty::ty::<C::Type>())
                .finish()
        }
    }

    #[repr(transparent)]
    pub struct ColumnValue<C: Column, V = <C as Column>::Type> {
        value: V,
        col: PhantomData<fn(C)>,
    }

    impl<C: Column, V> ColumnValue<C, V> {
        pub fn new(value: V) -> Self {
            Self {
                value,
                col: PhantomData,
            }
        }
    }

    impl<C, V> SpannerType for ColumnValue<C, V>
    where
        C: Column,
    {
        type Type = <C::Type as SpannerType>::Type;
        type Nullable = <C::Type as SpannerType>::Nullable;
    }

    impl<C, V> SpannerEncode for ColumnValue<C, V>
    where
        C: Column,
        V: SpannerEncode,
        V::SpannerType: SpannerType<
                Type = <C::Type as SpannerType>::Type,
                Nullable = <C::Type as SpannerType>::Nullable,
            >,
    {
        type Error = V::Error;
        type SpannerType = V::SpannerType;
        type Encoded = V::Encoded;

        #[inline]
        fn encode(self) -> Result<Self::Encoded, Self::Error> {
            self.value.encode()
        }

        #[inline]
        fn encode_to_value(self) -> Result<crate::Value, Self::Error> {
            self.value.encode_to_value()
        }
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
