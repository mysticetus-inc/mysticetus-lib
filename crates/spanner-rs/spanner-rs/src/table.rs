use crate::PrimaryKey;
use crate::insertable::Insertable;
use crate::queryable::Queryable;

/// Trait implemented on types that represent a single row in a spanner table.
///
/// Should be implemented by the [`spanner_rs_macros::Table`] proc macro, not manually.
pub trait Table: Insertable + Queryable {
    /// The name of the Spanner table.
    const NAME: &'static str;

    type Pk: PrimaryKey<Table = Self>;
}
