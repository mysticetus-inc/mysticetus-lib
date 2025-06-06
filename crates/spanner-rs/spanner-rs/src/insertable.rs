use std::marker::PhantomData;

use crate::error::ConvertError;
use crate::queryable::Row;

/// Trait for anything that can be inserted into a database.
pub trait Insertable: Row<ColumnName = &'static str> {
    /// Convert 'self' into a row for insertion into spanner.
    fn into_row(self) -> Result<crate::Row, ConvertError>;
}
