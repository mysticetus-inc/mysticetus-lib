use std::marker::PhantomData;

use crate::error::ConvertError;
use crate::{PrimaryKey, Row};

/// Trait implemented on types that represent a single row in a spanner table.
///
/// Should be implemented by the [`spanner_rs::row`] macro, not manually.
pub trait Table: crate::queryable::Row<ColumnName = &'static str> {
    /// The name of the Spanner table.
    const NAME: &'static str;

    type Pk: PrimaryKey<Table = Self>;

    /// Convert 'self' into a row for insertion into spanner.
    fn into_row(self) -> Result<Row, ConvertError>;
}

pub struct EncodedRow<T: Table> {
    column_indices: Option<&'static [usize]>,
    row: Row,
    marker: PhantomData<fn(T)>,
}

pub trait PartialTable: Sized {
    type Table: Table;

    fn column_indices(&self) -> &'static [usize];

    fn encode_row_into(self, row: &mut Row) -> Result<(), ConvertError>;

    fn into_encoded_row(self) -> Result<EncodedRow<Self::Table>, ConvertError> {
        let indices = self.column_indices();
        let mut row = Row(Vec::with_capacity(indices.len()));
        self.encode_row_into(&mut row)?;
        Ok(EncodedRow {
            column_indices: Some(indices),
            row,
            marker: PhantomData,
        })
    }
}
