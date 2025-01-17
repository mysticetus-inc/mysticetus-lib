use std::collections::HashMap;
use std::num::{NonZeroI64, NonZeroUsize};

use timestamp::Timestamp;

use super::TableReference;
use crate::builders::Unset;
use crate::builders::table_field_schema::TableFieldSchemaBuilder;
use crate::util;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table<S = Box<str>> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_reference: Option<TableReference<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub friendly_name: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<Box<str>, S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<TableSchema<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_partitioning: Option<TimePartitioning<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_partitioning: Option<RangePartitioning<S>>,
    #[serde(
        default = "Clustering::default",
        skip_serializing_if = "Clustering::is_empty"
    )]
    pub clustering: Clustering<S>,
    #[serde(default, skip_serializing_if = "util::is_false")]
    pub require_partition_filter: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::int64::optional"
    )]
    pub num_bytes: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::int64::optional"
    )]
    pub num_long_term_bytes: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::uint64::optional"
    )]
    pub num_rows: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::timestamp_ms::optional"
    )]
    pub expiration_time: Option<Timestamp>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::timestamp_ms::optional"
    )]
    pub last_modified_time: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub ty: Option<TableType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableType {
    Table,
    View,
    External,
    MaterializedView,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema<S> {
    pub fields: Vec<TableFieldSchema<S>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema<S> {
    pub name: S,
    #[serde(rename = "type")]
    pub ty: FieldType,
    pub mode: FieldMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<S>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::int64::optional"
    )]
    pub max_length: Option<NonZeroUsize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::int64::optional"
    )]
    pub precision: Option<NonZeroI64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "util::int64::optional"
    )]
    pub scale: Option<NonZeroI64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rounding_mode: Option<RoundingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value_expression: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_element_type: Option<RangeElementType>,
}

impl<S> TableFieldSchema<S> {
    pub const fn builder(name: S) -> TableFieldSchemaBuilder<S, Unset> {
        TableFieldSchemaBuilder::new(name)
    }

    pub const fn new(name: S, ty: FieldType, mode: FieldMode) -> Self {
        Self {
            name,
            ty,
            mode,
            rounding_mode: None,
            range_element_type: None,
            max_length: None,
            description: None,
            default_value_expression: None,
            scale: None,
            precision: None,
        }
    }

    pub fn map_as_ref<T: ?Sized>(&self) -> TableFieldSchema<&T>
    where
        S: AsRef<T>,
    {
        self.map(AsRef::as_ref)
    }

    pub fn map<'a, T: 'a>(&'a self, mut map_fn: impl FnMut(&'a S) -> T) -> TableFieldSchema<T> {
        TableFieldSchema {
            name: map_fn(&self.name),
            ty: self.ty,
            mode: self.mode,
            description: self.description.as_ref().map(&mut map_fn),
            max_length: self.max_length,
            precision: self.precision,
            scale: self.scale,
            rounding_mode: self.rounding_mode,
            default_value_expression: self.default_value_expression.as_ref().map(map_fn),
            range_element_type: self.range_element_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoundingMode {
    /// Unspecified will default to using ROUND_HALF_AWAY_FROM_ZERO
    RoundingModeUnspecified,
    /// ROUND_HALF_AWAY_FROM_ZERO rounds half values away from zero when applying
    /// precision and scale upon writing of NUMERIC and BIGNUMERIC values.
    /// For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1 1.5, 1.6, 1.7, 1.8, 1.9 => 2
    RoundHalfAwayFromZero,
    /// ROUND_HALF_EVEN rounds half values to the nearest even value when applying
    /// precision and scale upon writing of NUMERIC and BIGNUMERIC values.
    /// For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1 1.5 => 2 1.6, 1.7, 1.8, 1.9 => 2 2.5 => 2
    RoundHalfEven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RangeElementType {
    Date,
    DateTime,
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FieldMode {
    Nullable,
    Repeated,
    Required,
}

impl FieldMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Nullable => "Nullable",
            Self::Repeated => "Repeated",
            Self::Required => "Required",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FieldType {
    String,
    Bytes,
    #[serde(alias = "INT64")]
    Integer,
    #[serde(alias = "FLOAT64")]
    Float,
    #[serde(alias = "BOOLEAN")]
    Bool,
    Timestamp,
    Date,
    Time,
    DateTime,
    Geography,
    Numeric,
    BigNumeric,
    Json,
    #[serde(alias = "STRUCT")]
    Record,
    Range,
    Interval,
}

impl FieldType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Bytes => "Bytes",
            Self::Integer => "Int64",
            Self::Float => "Float64",
            Self::Bool => "Bool",
            Self::Timestamp => "Timestamp",
            Self::Date => "Date",
            Self::Time => "Time",
            Self::DateTime => "DateTime",
            Self::Geography => "Geography",
            Self::Numeric => "Numeric",
            Self::BigNumeric => "BigNumeric",
            Self::Json => "Json",
            Self::Record => "Struct",
            Self::Range => "Range",
            Self::Interval => "Interval",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimePartitioning<S> {
    #[serde(rename = "type")]
    pub ty: TimePartitioningType,
    #[serde(
        default,
        with = "util::duration_ms::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<timestamp::Duration>,
    pub field: Option<S>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RangePartitioning<S> {
    pub field: S,
    pub range: RangePartition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RangePartition {
    pub start: i64,
    pub end: i64,
    pub interval: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimePartitioningType {
    Day,
    Hour,
    Month,
    Year,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clustering<S> {
    pub fields: Vec<S>,
}

impl<S> Clustering<S> {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl<S> Default for Clustering<S> {
    fn default() -> Self {
        Self { fields: vec![] }
    }
}
