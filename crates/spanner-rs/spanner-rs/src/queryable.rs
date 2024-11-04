use generic_array::{ArrayLength, GenericArray};

use crate::column::Column;
use crate::results::RawRow;

pub trait Queryable: Sized {
    type NumColumns: ArrayLength;

    const COLUMNS: GenericArray<Column<'static>, Self::NumColumns>;

    fn from_row(row: RawRow<'_, Self::NumColumns>) -> crate::Result<Self>;
}
