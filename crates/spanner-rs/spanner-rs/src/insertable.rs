use crate::error::ConvertError;
use crate::queryable::Queryable;

/// Trait for anything that can be inserted into a database.
pub trait Insertable: Queryable {
    /// Convert 'self' into a row for insertion into spanner.
    fn into_row(self) -> Result<crate::Row, ConvertError>;
}
