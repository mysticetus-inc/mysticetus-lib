use generic_array::{ArrayLength, GenericArray};

use crate::column::Column;
use crate::results::RawRow;
use crate::ty::SpannerType;

pub trait Queryable: Sized {
    type NumColumns: ArrayLength;

    const COLUMNS: GenericArray<Column<'static>, Self::NumColumns>;

    fn from_row(row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self>;
}

pub trait Columns: Tuple {}

pub trait Col {
    type Type: SpannerType;

    fn name(&self) -> Option<&str>;

    fn index(&self) -> usize;
}

pub trait Tuple {
    type Arity: typenum::Unsigned;
}

pub trait TupleElement<Index: typenum::Unsigned>: Tuple {
    type Value;

    fn get_ref(&self) -> &Self::Value;
    fn get_mut(&mut self) -> &mut Self::Value;
}

impl Tuple for () {
    type Arity = typenum::U0;
}

impl<T> Tuple for (T,) {
    type Arity = typenum::U1;
}

impl<T> TupleElement<typenum::U0> for (T,) {
    type Value = T;

    #[inline]
    fn get_ref(&self) -> &Self::Value {
        &self.0
    }
    #[inline]
    fn get_mut(&mut self) -> &mut Self::Value {
        &mut self.0
    }
}

macro_rules! impl_tuple {
    () => {};
}
