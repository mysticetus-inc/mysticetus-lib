use std::marker::PhantomData;
use std::num::NonZeroU64;

use serde::de;

use crate::table::TableSchema;

pub struct RowsVisitor<'a, S, Row> {
    pub(super) schema: &'a TableSchema<S>,
    pub(super) total_rows: Option<u64>,
    pub(super) request_limit: Option<NonZeroU64>,
    pub(super) _marker: PhantomData<fn(Row)>,
}

impl<S, Row> RowsVisitor<'_, S, Row> {
    fn estimate_capacity(&self) -> Option<usize> {
        match (self.total_rows, self.request_limit) {
            (Some(total), Some(limit)) => Some(limit.get().min(total) as usize),
            (Some(total), None) => Some(total as usize),
            (None, Some(limit)) => Some(limit.get() as usize),
            (None, None) => None,
        }
    }
}

impl<'de, S, Row> de::DeserializeSeed<'de> for RowsVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Vec<Row>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, S, Row> de::Visitor<'de> for RowsVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Vec<Row>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of encoded rows")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let est_capacity = self
            .estimate_capacity()
            .or_else(|| seq.size_hint())
            .unwrap_or(32);

        let mut rows = Vec::with_capacity(est_capacity);

        let mut row_visitor = super::row::RowVisitor {
            schema: self.schema,
            _marker: PhantomData,
        };

        while let Some(elem) = seq.next_element_seed(&mut row_visitor)? {
            rows.push(elem);
        }

        Ok(rows)
    }
}
