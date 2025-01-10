use std::marker::PhantomData;
use std::num::NonZeroU64;

use serde::de;
use serde_json::value::RawValue;

use crate::query::rows::RowsVisitor;
use crate::table::TableSchema;

#[derive(Debug, Clone)]
pub struct QueryResults<Row, S = Box<str>> {
    pub schema: Option<TableSchema<S>>,
    pub total_rows: Option<u64>,
    pub page_token: Option<S>,
    pub job_complete: bool,
    pub rows: Vec<Row>,
}

impl<'de, Row, S> serde::Deserialize<'de> for QueryResults<Row, S>
where
    Row: serde::Deserialize<'de>,
    S: serde::Deserialize<'de> + AsRef<str>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(QueryResultsVisitor {
            request_limit: None,
            _marker: PhantomData,
        })
    }
}

struct QueryResultsVisitor<Row, S> {
    request_limit: Option<NonZeroU64>,
    _marker: PhantomData<fn(Row, S)>,
}

impl<'de, Row, S> de::Visitor<'de> for QueryResultsVisitor<Row, S>
where
    Row: serde::Deserialize<'de>,
    S: serde::Deserialize<'de> + AsRef<str>,
{
    type Value = QueryResults<Row, S>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a query response")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        enum RowsState<'de, Row> {
            Empty,
            // This assumes we're deserializing from something completely in memory,
            // which should always be the case (since its deserialing from a buffer
            // returned by an http request, not reading from anything)
            WaitingForSchema(&'de RawValue),
            Decoded(Vec<Row>),
        }

        let mut schema: Option<TableSchema<S>> = None;
        let mut total_rows: Option<u64> = None;
        let mut page_token: Option<S> = None;
        let mut job_complete: Option<bool> = None;
        let mut rows = RowsState::<'de, Row>::Empty;

        macro_rules! duplicate_field {
            ($field:literal) => {
                return Err(de::Error::duplicate_field($field))
            };
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        enum Fields {
            Schema,
            TotalRows,
            PageToken,
            JobComplete,
            Rows,
            #[serde(other)]
            Other,
        }

        while let Some(field) = map.next_key()? {
            match field {
                Fields::Schema if schema.is_some() => duplicate_field!("schema"),
                Fields::Schema => schema = Some(map.next_value()?),
                Fields::TotalRows if total_rows.is_some() => duplicate_field!("totalRows"),
                Fields::TotalRows => {
                    total_rows = Some(map.next_value_seed(crate::util::Uint64ValueVisitor)?);
                }
                Fields::PageToken if page_token.is_some() => duplicate_field!("pageToken"),
                Fields::PageToken => page_token = Some(map.next_value()?),
                Fields::JobComplete if job_complete.is_some() => duplicate_field!("jobComplete"),
                Fields::JobComplete => job_complete = Some(map.next_value()?),
                Fields::Rows if !matches!(rows, RowsState::Empty) => duplicate_field!("rows"),
                Fields::Rows => {
                    rows = match schema {
                        Some(ref schema) => {
                            RowsState::Decoded(map.next_value_seed(RowsVisitor {
                                schema,
                                request_limit: self.request_limit,
                                total_rows,
                                _marker: PhantomData,
                            })?)
                        }
                        None => RowsState::WaitingForSchema(map.next_value()?),
                    };
                }
                Fields::Other => _ = map.next_value::<de::IgnoredAny>()?,
            }
        }

        let rows = match rows {
            RowsState::Empty => Vec::new(),
            RowsState::Decoded(rows) => rows,
            RowsState::WaitingForSchema(raw_rows) => {
                let Some(ref schema) = schema else {
                    return Err(de::Error::missing_field("schema"));
                };

                serde::Deserializer::deserialize_seq(raw_rows, RowsVisitor {
                    schema,
                    request_limit: self.request_limit,
                    total_rows,
                    _marker: PhantomData,
                })
                .map_err(de::Error::custom)?
            }
        };

        Ok(QueryResults {
            schema,
            total_rows,
            page_token,
            job_complete: job_complete.unwrap_or(false),
            rows,
        })
    }
}
