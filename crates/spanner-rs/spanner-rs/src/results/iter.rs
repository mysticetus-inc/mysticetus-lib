use protos::protobuf::ListValue;
use protos::spanner;

use super::stats::QueryStats;
use super::{FieldIndex, RawRow};
use crate::Table;

pub struct ResultIter<T: Table> {
    field_index: FieldIndex<T::NumColumns>,
    rows: std::vec::IntoIter<ListValue>,
    stats: Option<spanner::ResultSetStats>,
}

impl<T: Table> ResultIter<T> {
    pub(crate) fn into_parts(self) -> (FieldIndex<T::NumColumns>, std::vec::IntoIter<ListValue>) {
        (self.field_index, self.rows)
    }

    #[inline]
    pub(crate) fn from_result_set(result_set: spanner::ResultSet) -> crate::Result<Self> {
        let field_index = FieldIndex::from_result_set_meta::<T>(result_set.metadata)?;

        Ok(Self {
            field_index,
            rows: result_set.rows.into_iter(),
            stats: result_set.stats,
        })
    }

    #[inline]
    pub(crate) fn from_parts(
        field_index: FieldIndex<T::NumColumns>,
        rows: Vec<ListValue>,
        stats: Option<spanner::ResultSetStats>,
    ) -> Self {
        Self {
            field_index,
            rows: rows.into_iter(),
            stats,
        }
    }

    #[inline]
    pub fn take_query_stats(&mut self) -> Option<QueryStats> {
        self.stats
            .take()
            .and_then(|raw| raw.query_stats)
            .map(QueryStats::from_struct)
    }

    pub fn take_raw_stats(&mut self) -> Option<spanner::ResultSetStats> {
        self.stats.take()
    }

    pub fn stats(&self) -> Option<&spanner::ResultSetStats> {
        self.stats.as_ref()
    }
}

#[inline]
fn handle_row<T: Table>(index: &FieldIndex<T::NumColumns>, row: ListValue) -> crate::Result<T> {
    if T::COLUMNS.len() != row.values.len() {
        return Err(crate::Error::MismatchedColumnCount {
            expected: T::COLUMNS.len(),
            found: row.values.len(),
        });
    }

    let raw = RawRow::new(index, row.values);

    T::from_row(raw).map_err(Into::into)
}

impl<T: Table> Iterator for ResultIter<T> {
    type Item = crate::Result<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rows
            .next()
            .map(|row| handle_row(&self.field_index, row))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rows.len();

        (len, Some(len))
    }
}

impl<T: Table> ExactSizeIterator for ResultIter<T> {}

impl<T: Table> DoubleEndedIterator for ResultIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rows
            .next_back()
            .map(|row| handle_row(&self.field_index, row))
    }
}

impl<T: Table> std::iter::FusedIterator for ResultIter<T> {}
