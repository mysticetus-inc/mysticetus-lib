use std::collections::HashMap;
use std::marker::PhantomData;
use std::num::NonZeroU64;

pub use query_string::{QueryString, query};
use serde::de::{self, IntoDeserializer};
use serde_json::value::RawValue;
use uuid::Uuid;

mod results;
mod row;
mod rows;
mod value;

pub use results::QueryResults;

use crate::job::{JobCreationMode, JobReference};
use crate::table::{FieldMode, FieldType, TableFieldSchema, TableSchema};
use crate::{DatasetReference, ErrorProto};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest<S = Box<str>> {
    pub query: QueryString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<NonZeroU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset: Option<DatasetReference<S>>,
    #[serde(
        rename = "timeoutMs",
        serialize_with = "serialize_timeout_ms",
        skip_serializing_if = "Option::is_none"
    )]
    pub timeout: Option<timestamp::Duration>,
    #[serde(skip_serializing_if = "crate::util::is_false")]
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_query_cache: Option<bool>,
    pub use_legacy_sql: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_mode: Option<ParameterMode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query_parameters: Vec<QueryParameter<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<S>,
    #[serde(skip_serializing_if = "DataFormatOptions::is_default")]
    pub format_options: DataFormatOptions,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connection_properties: Vec<ConnectionProperty<S>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<Box<str>, S>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "crate::util::uint64::optional"
    )]
    pub maximum_bytes_billed: Option<NonZeroU64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
    #[serde(skip_serializing_if = "crate::util::is_false")]
    pub create_session: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_creation_mode: Option<JobCreationMode>,
}

mod query_string {
    /// Wrapper around a static string, used to statically prevent
    /// sql injection vectors.
    ///
    /// We do this by making this type impossible to create without
    /// an unsafe (and hidden) function call, that has its invariants
    /// enforced by a macro (by ensuring the string is a literal)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    #[repr(transparent)]
    pub struct QueryString(&'static str);

    impl QueryString {
        /// SAFETY: Calling this with a leaked string
        /// gives a path to doing sql injection, therefore
        /// this should only be called with a literal sql string
        /// (which is enforced by the [`query`] macro)
        #[doc(hidden)]
        #[inline(always)]
        pub const unsafe fn __new(s: &'static str) -> Self {
            Self(s)
        }
    }

    #[macro_export]
    macro_rules! query {
        ($sql:literal) => {{
            /// SAFETY: this macro ensures we're being passed a literal
            unsafe {
                QueryString::__new($sql)
            }
        }};
    }

    pub use query;
}

impl<S> QueryRequest<S> {
    pub fn new(query: QueryString) -> Self {
        Self {
            query,
            labels: HashMap::new(),
            max_results: None,
            default_dataset: None,
            timeout: None,
            dry_run: false,
            use_query_cache: None,
            use_legacy_sql: false,
            parameter_mode: None,
            query_parameters: Vec::new(),
            location: None,
            format_options: DataFormatOptions::default(),
            connection_properties: Vec::new(),
            maximum_bytes_billed: None,
            request_id: None,
            create_session: false,
            job_creation_mode: None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse<Row, S: AsRef<str> = Box<str>> {
    pub job_reference: Option<JobReference<S>>,
    #[serde(deserialize_with = "deserialize_job_creation_reason")]
    pub job_creation_reason: Option<JobCreationReason>,
    pub query_id: Option<S>,
    #[serde(default, with = "crate::util::int64::optional")]
    pub total_bytes_processed: Option<i64>,
    #[serde(default = "Vec::new")]
    pub errors: Vec<ErrorProto<S>>,
    #[serde(default)]
    pub cache_hit: bool,
    #[serde(default, with = "crate::util::int64::optional")]
    pub num_dml_affected_rows: Option<i64>,
    pub session_info: Option<SessionInfo<S>>,
    pub dml_stats: Option<DmlStats>,
    #[serde(flatten)]
    pub results: QueryResults<Row, S>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo<S = Box<str>> {
    pub session_id: S,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmlStats {
    #[serde(with = "crate::util::int64")]
    pub inserted_row_count: u64,
    #[serde(with = "crate::util::int64")]
    pub deleted_row_count: u64,
    #[serde(with = "crate::util::int64")]
    pub updated_row_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobCreationReason {
    Requested,
    LongRunning,
    LargeResults,
    Other,
}

fn deserialize_job_creation_reason<'de, D>(
    deserializer: D,
) -> Result<Option<JobCreationReason>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    struct Wrapper {
        code: JobCreationReason,
    }

    match serde::Deserialize::deserialize(deserializer)? {
        Some(Wrapper { code }) => Ok(Some(code)),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionProperty<S = Box<str>> {
    key: S,
    value: S,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DataFormatOptions {
    #[serde(skip_serializing_if = "crate::util::is_false")]
    use_int64_timestamp: bool,
}

impl DataFormatOptions {
    pub fn is_default(&self) -> bool {
        self.use_int64_timestamp == false
    }
}

fn serialize_timeout_ms<S>(
    opt: &Option<timestamp::Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match opt {
        None => serializer.serialize_none(),
        Some(timeout) => serializer.serialize_some(&timeout.millis()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParameterMode {
    Positional,
    Named,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct QueryParameter<S = Box<str>> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<S>,
    parameter_type: QueryParameterType,
    parameter_value: QueryParameterValue,
}

impl<S> QueryParameter<S> {
    pub fn scalar(ty: FieldType, value: QueryParameterValue) -> Self {
        Self {
            name: None,
            parameter_type: QueryParameterType::Scalar(ty),
            parameter_value: value,
        }
    }

    pub fn named(mut self, name: impl Into<S>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParameterType {
    Scalar(FieldType),
    Array {
        element_type: Box<QueryParameterType>,
    },
    Struct {
        fields: Vec<StructField>,
    },
    Range {
        element_type: Box<QueryParameterType>,
    },
}

macro_rules! to_unit {
    ($i:ident) => {
        ()
    };
}

macro_rules! serialize_map {
    (
        $serializer:expr;
        $($key:ident => $value:expr),* $(,)?
    ) => {{
        use serde::ser::SerializeMap;

        const LEN_HELPER: &[()] = &[$(to_unit!($key)),*];

        let mut map = $serializer.serialize_map(Some(LEN_HELPER.len()))?;

        $(
            map.serialize_entry(stringify!($key), &$value)?;
        )*

        map.end()
    }};
}

impl serde::Serialize for QueryParameterType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Scalar(ty) => serialize_map!(serializer; type => ty),
            Self::Array { element_type } => serialize_map! {
                serializer;
                type => "ARRAY",
                arrayType => element_type,
            },
            Self::Struct { fields } => serialize_map! {
                serializer;
                type => "STRUCT",
                structTypes => fields,
            },
            Self::Range { element_type } => serialize_map! {
                serializer;
                type => "RANGE",
                rangeElementType => element_type,
            },
        }
    }
}

impl<'de> serde::Deserialize<'de> for QueryParameterType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawQueryParamType {
            #[serde(rename = "type")]
            ty: FieldType,
            array_type: Option<QueryParameterType>,
            struct_types: Option<Vec<StructField>>,
            range_element_type: Option<QueryParameterType>,
        }

        let RawQueryParamType {
            ty,
            array_type,
            struct_types,
            range_element_type,
        } = serde::Deserialize::deserialize(deserializer)?;

        match (array_type, struct_types, range_element_type) {
            // valid combinations
            (Some(element_type), None, None) => Ok(Self::Array {
                element_type: Box::new(element_type),
            }),
            (None, Some(fields), None) => Ok(Self::Struct { fields }),
            (None, None, Some(element_type)) => Ok(Self::Range {
                element_type: Box::new(element_type),
            }),
            (None, None, None) => Ok(Self::Scalar(ty)),
            // invalid combinations
            (Some(_), Some(_), Some(_)) => Err(serde::de::Error::custom(
                "expected one of 'arrayType', 'structType' and 'rangeElementType', got all 3",
            )),
            (None, Some(_), Some(_)) => Err(serde::de::Error::custom(
                "expected one of 'structType' and 'rangeElementType', got both",
            )),
            (Some(_), None, Some(_)) => Err(serde::de::Error::custom(
                "expected one of 'arrayType' and 'rangeElementType', got both",
            )),
            (Some(_), Some(_), None) => Err(serde::de::Error::custom(
                "expected one of 'arrayType' and 'structType', got both",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct StructField {
    name: Option<Box<str>>,
    #[serde(rename = "type")]
    ty: QueryParameterType,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParameterValue {
    Scalar(serde_json::Value),
    Array(Vec<serde_json::Value>),
    Struct(HashMap<Box<str>, serde_json::Value>),
    Range(Box<Range<QueryParameterValue>>),
}

impl serde::Serialize for QueryParameterValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Scalar(scalar) => serialize_map!(serializer; value => scalar),
            Self::Array(array) => serialize_map!(serializer; arrayValues => array),
            Self::Struct(fields) => serialize_map!(serializer; structValues => fields),
            Self::Range(range) => serialize_map!(serializer; rangeValue => range),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range<T> {
    StartAt(T),
    EndAt(T),
    Bounded { start: T, end: T },
}

impl<'de, T> serde::Deserialize<'de> for Range<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RawRange<T2> {
            start: Option<T2>,
            end: Option<T2>,
        }

        let RawRange { start, end } = RawRange::<T>::deserialize(deserializer)?;

        match (start, end) {
            (Some(start), Some(end)) => Ok(Self::Bounded { start, end }),
            (Some(start), None) => Ok(Self::StartAt(start)),
            (None, Some(end)) => Ok(Self::EndAt(end)),
            (None, None) => Err(serde::de::Error::custom(
                "expected one or both of 'start' and 'end', got neither",
            )),
        }
    }
}

impl<T: serde::Serialize> serde::Serialize for Range<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Bounded { start, end } => serialize_map!(serializer; start => start, end => end),
            Self::StartAt(start) => serialize_map!(serializer; start => start),
            Self::EndAt(end) => serialize_map!(serializer; end => end),
        }
    }
}

struct FieldSeed<'a, S, V> {
    column: &'a TableFieldSchema<S>,
    seed: V,
}

impl<'de, S, V> de::DeserializeSeed<'de> for FieldSeed<'_, S, V>
where
    V: de::DeserializeSeed<'de>,
    S: AsRef<str>,
{
    type Value = V::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}
impl<'de, S, V> de::Visitor<'de> for FieldSeed<'_, S, V>
where
    V: de::DeserializeSeed<'de>,
    S: AsRef<str>,
{
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an object containing {'v': ")?;
        write!(formatter, "{}}}", std::any::type_name::<V::Value>())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        #[derive(serde::Deserialize)]
        enum Field {
            #[serde(rename = "v")]
            V,
            #[serde(other)]
            Other,
        }

        let Self { column, seed } = self;

        let mut seed = Some(seed);
        let mut value = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::V => match seed.take() {
                    Some(seed) => value = Some(map.next_value_seed(seed)?),
                    None => return Err(de::Error::duplicate_field("v")),
                },
                Field::Other => _ = map.next_value::<de::IgnoredAny>()?,
            }
        }

        value.ok_or_else(|| de::Error::missing_field("v"))
    }
}

struct FieldVisitor<'a, S, V> {
    column: &'a TableFieldSchema<S>,
    seed: V,
}

impl<'de, S, V> de::DeserializeSeed<'de> for FieldVisitor<'_, S, V>
where
    S: AsRef<str>,
    V: de::DeserializeSeed<'de>,
{
    type Value = V::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.column.mode {
            FieldMode::Nullable => deserializer.deserialize_option(self),
            FieldMode::Required => match self.column.ty {
                FieldType::String => deserializer.deserialize_string(self),
                FieldType::Bytes => deserializer.deserialize_byte_buf(self),
                FieldType::Integer => deserializer.deserialize_i64(self),
                FieldType::Float => deserializer.deserialize_f64(self),
                FieldType::Bool => deserializer.deserialize_bool(self),
                FieldType::Timestamp => deserializer.deserialize_any(self),
                FieldType::Date => deserializer.deserialize_any(self),
                FieldType::Time => deserializer.deserialize_any(self),
                FieldType::DateTime => deserializer.deserialize_any(self),
                FieldType::Geography => deserializer.deserialize_any(self),
                FieldType::Numeric => deserializer.deserialize_any(self),
                FieldType::BigNumeric => deserializer.deserialize_any(self),
                FieldType::Json => deserializer.deserialize_map(self),
                FieldType::Record => deserializer.deserialize_map(self),
                FieldType::Range => deserializer.deserialize_any(self),
                FieldType::Interval => deserializer.deserialize_any(self),
            },
            FieldMode::Repeated => deserializer.deserialize_seq(self),
        }
    }
}

impl<'de, S, V> de::Visitor<'de> for FieldVisitor<'_, S, V>
where
    S: AsRef<str>,
    V: de::DeserializeSeed<'de>,
{
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an encoded value")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        todo!()
    }
}
