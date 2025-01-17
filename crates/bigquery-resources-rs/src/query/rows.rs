use std::marker::PhantomData;
use std::num::NonZeroU64;

use serde::de;

use crate::table::TableSchema;

pub struct RowsVisitor<'a, S, Row> {
    pub schema: &'a TableSchema<S>,
    pub total_rows: Option<u64>,
    pub request_limit: Option<NonZeroU64>,
    pub _marker: PhantomData<fn(Row)>,
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
        match serde::Deserializer::deserialize_seq(
            path_aware_serde::Deserializer::new(deserializer),
            self,
        ) {
            Ok(value) => Ok(value),
            Err(error) => {
                let (inner_error, path) = error.into_inner();
                // if the path is non-empty, return the original error with more context
                if let Some(path) = path {
                    if !path.is_empty() {
                        return Err(de::Error::custom(format!("{path}: {inner_error}")));
                    }
                }

                // otherwise, just return the original error
                Err(inner_error)
            }
        }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use path_aware_serde::DeserializeSeedExt;

    use super::*;
    use crate::table::{TableFieldSchema, TableSchema};

    #[derive(Debug, Clone, serde::Deserialize)]
    struct ExampleRow {
        string: Box<str>,
        optional_string: Option<Box<str>>,
        optional_string2: Option<Box<str>>,
        integer: i64,
        float: f64,
        timestamp: timestamp::Timestamp,
        json: BTreeMap<Box<str>, serde_json::Value>,
        optional_json: Option<BTreeMap<Box<str>, serde_json::Value>>,
    }

    impl ExampleRow {
        fn schema() -> TableSchema<&'static str> {
            let fields = vec![
                TableFieldSchema::builder("string").string().required(),
                TableFieldSchema::builder("optional_string")
                    .string()
                    .nullable(),
                TableFieldSchema::builder("optional_string2")
                    .string()
                    .nullable(),
                TableFieldSchema::builder("integer").int().required(),
                TableFieldSchema::builder("float").float().required(),
                TableFieldSchema::builder("timestamp")
                    .timestamp()
                    .required(),
                TableFieldSchema::builder("json").json().required(),
                TableFieldSchema::builder("optional_json").json().nullable(),
            ];

            TableSchema { fields }
        }
    }

    const JSON_EXAMPLE: &str = r#"[{
        "f": [
            {"v": "test"},
            {"v": null},
            {"v": "optional but valid"},
            {"v": "1234"},
            {"v": "1e6"},
            {"v": "1.724e9"},
            {"v": "{}"},
            {"v": "{\"test\": true}"}
        ]
    }]"#;

    #[test]
    fn test_deserialize() -> Result<(), Box<dyn std::error::Error>> {
        let schema = ExampleRow::schema();

        let visitor = RowsVisitor {
            schema: &schema,
            total_rows: Some(1),
            request_limit: None,
            _marker: std::marker::PhantomData,
        };

        let mut de = serde_json::de::Deserializer::from_str(JSON_EXAMPLE);

        let mut deserialized_rows: Vec<ExampleRow> =
            visitor.deserialize_seed_path_aware(&mut de)?;

        assert_eq!(deserialized_rows.len(), 1);

        let row = deserialized_rows.pop().unwrap();

        println!("{row:#?}");

        Ok(())
    }
}
