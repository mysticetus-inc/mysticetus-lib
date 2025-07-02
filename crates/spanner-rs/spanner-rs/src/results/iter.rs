use protos::protobuf::ListValue;
use protos::spanner;

use super::stats::QueryStats;
use super::{FieldIndex, RawRow};
use crate::queryable::Queryable;

pub struct ResultIter<T: Queryable> {
    field_index: FieldIndex<T::NumColumns>,
    rows: std::vec::IntoIter<ListValue>,
    stats: Option<spanner::ResultSetStats>,
}

impl<T: Queryable> std::fmt::Debug for ResultIter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultIter")
            .field("stats", &self.stats.as_ref().map(DebugResultSetStats))
            .field("field_index", &self.field_index)
            .field("rows", &self.rows.as_slice())
            .finish()
    }
}

/// Wrapper around the raw proto type that formats things a bit more nicely, since all
/// the raw protobuf Structs/Values add way more noise since theyre all so nested.
struct DebugResultSetStats<'a>(&'a spanner::ResultSetStats);

struct DebugPlanNodes<'a>(&'a [spanner::PlanNode]);

impl std::fmt::Debug for DebugPlanNodes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(DebugPlanNode))
            .finish()
    }
}

struct DebugPlanNode<'a>(&'a spanner::PlanNode);

impl std::fmt::Debug for DebugPlanNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::value::fmt_helpers::DebugMap;

        let mut f = f.debug_struct("PlanNode");

        f.field("index", &self.0.index)
            .field("kind", &self.0.kind());

        if !self.0.display_name.is_empty() {
            f.field("display_name", &self.0.display_name);
        }

        if let Some(ref short_repr) = self.0.short_representation {
            if !short_repr.description.is_empty() || !short_repr.subqueries.is_empty() {
                f.field("short_repr", &DebugShortRepr(short_repr));
            }
        }

        if let Some(ref exe_stats) = self.0.execution_stats {
            if !exe_stats.fields.is_empty() {
                f.field("execution_stats", &DebugMap(&exe_stats.fields));
            }
        }

        if let Some(ref meta) = self.0.metadata {
            if !meta.fields.is_empty() {
                f.field("metadata", &DebugMap(&meta.fields));
            }
        }

        if !self.0.child_links.is_empty() {
            f.field("child_links", &DebugChildLinks(&self.0.child_links));
        }

        f.finish()
    }
}

struct DebugChildLinks<'a>(&'a [spanner::plan_node::ChildLink]);

impl std::fmt::Debug for DebugChildLinks<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(DebugChildLink))
            .finish()
    }
}
struct DebugChildLink<'a>(&'a spanner::plan_node::ChildLink);

impl std::fmt::Debug for DebugChildLink<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("ChildLink");

        f.field("child_index", &self.0.child_index);

        if !self.0.r#type.is_empty() {
            f.field("type", &self.0.r#type);
        }

        if !self.0.variable.is_empty() {
            f.field("variable", &self.0.variable);
        }

        f.finish()
    }
}

struct DebugShortRepr<'a>(&'a spanner::plan_node::ShortRepresentation);

impl std::fmt::Debug for DebugShortRepr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("ShortRepr");

        if !self.0.description.is_empty() {
            f.field("description", &self.0.description);
        }

        if !self.0.subqueries.is_empty() {
            f.field("subqueries", &self.0.subqueries);
        }

        f.finish()
    }
}

impl std::fmt::Debug for DebugResultSetStats<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::value::fmt_helpers::DebugMap;

        let spanner::ResultSetStats {
            query_stats,
            row_count,
            query_plan,
        } = &self.0;

        let mut f = f.debug_struct("ResultSetStats");

        if let Some(stats) = query_stats {
            if !stats.fields.is_empty() {
                f.field("stats", &DebugMap(&stats.fields));
            }
        }

        if let Some(row_count) = row_count {
            f.field("row_count", row_count);
        }

        if let Some(qp) = query_plan {
            if !qp.plan_nodes.is_empty() {
                f.field("query_plan", &DebugPlanNodes(&qp.plan_nodes));
            }
        }

        f.finish()
    }
}

impl<T: Queryable> ResultIter<T> {
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
fn handle_row<T: Queryable>(index: &FieldIndex<T::NumColumns>, row: ListValue) -> crate::Result<T> {
    if T::COLUMNS.len() != row.values.len() {
        return Err(crate::Error::MismatchedColumnCount {
            expected: T::COLUMNS.len(),
            found: row.values.len(),
        });
    }

    let raw = RawRow::new(index, row.values);

    T::from_row(raw).map_err(Into::into)
}

impl<T: Queryable> Iterator for ResultIter<T> {
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

impl<T: Queryable> ExactSizeIterator for ResultIter<T> {}

impl<T: Queryable> DoubleEndedIterator for ResultIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rows
            .next_back()
            .map(|row| handle_row(&self.field_index, row))
    }
}

impl<T: Queryable> std::iter::FusedIterator for ResultIter<T> {}
