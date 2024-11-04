use crate::PrimaryKey;
use crate::insertable::Insertable;
use crate::queryable::Queryable;

/*
// old version for reference
pub trait Table: Sized + 'static {
    /// The name of the Spanner table.
    const NAME: &'static str;

    /// The columns of the spanner table. In all requests, this determines
    /// the column order of the rows returned (as they get returned in lists of values).
    const COLS: &'static [&'static dyn Column<Self>];

    /// The number of columns.
    const N_COLS: usize = Self::COLS.len();

    /// The proc macro generated primary key type.
    type PrimaryKey: PrimaryKey<Self>;

    /// Converts a row into its primary key, without cloning.
    fn into_pk(self) -> Self::PrimaryKey;

    /// Const-generic indexing into Self::COLS. Will panic if out of bounds.
    fn col_at_index<const INDEX: usize>() -> &'static dyn Column<Self> {
        Self::COLS[INDEX]
    }

    /// Builds a Primary key for a row, cloning only the fields it needs.
    fn to_pk(&self) -> Self::PrimaryKey;

    /// From the given field information, extract values from a row to create 'Self'.
    fn from_row(fields: &[Field], row: Row) -> Result<Self, ConvertError>;

    /// Convert 'self' to a row, for writing to spanner.
    fn to_row(self) -> Result<Row, ConvertError>;


}
*/

/// Trait implemented on types that represent a single row in a spanner table.
///
/// Should be implemented by the [`spanner_rs_macros::Table`] proc macro, not manually.
pub trait Table: Insertable {
    /// The name of the Spanner table.
    const NAME: &'static str;

    type Pk: PrimaryKey<Table = Self>;

    fn sql_select(force_index: Option<&str>) -> String {
        let cap = Self::COLUMNS
            .iter()
            .map(|col| col.name.len() + 2)
            .sum::<usize>();

        let mut dst = String::with_capacity(2 * cap);

        dst.push_str("SELECT ");
        for col in Self::COLUMNS.iter() {
            dst.push_str(col.name);

            if col.index + 1 < Self::COLUMNS.len() {
                dst.push_str(", ");
            }
        }

        dst.push_str(" FROM ");
        dst.push_str(Self::NAME);
        if let Some(index) = force_index {
            dst.push_str("@{FORCE_INDEX=");
            dst.push_str(index);
            dst.push_str("} ");
        } else {
            dst.push_str(" ");
        }

        dst
    }
}

// TODO
pub trait InterleavedTable: Table {
    type Parent: Table;
}
