use crate::resources::TableReference;
use crate::util;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationExtract<S = Box<str>> {
    pub destination_uris: Vec<S>,
    #[serde(flatten)]
    pub kind: ExtractKind<S>,
}

impl<S> JobConfigurationExtract<S> {
    pub fn new(destination_uris: impl Into<Vec<S>>, kind: impl Into<ExtractKind<S>>) -> Self {
        Self {
            destination_uris: destination_uris.into(),
            kind: kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ExtractKind<S = Box<str>> {
    Table(TableExtract<S>),
    Model(ModelExtract),
}

// empty enum to indicate not implemented yet
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ModelExtract {}

impl<S> From<TableExtract<S>> for ExtractKind<S> {
    fn from(value: TableExtract<S>) -> Self {
        Self::Table(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableExtract<S = Box<str>> {
    pub source_table: TableReference<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<Compression>,
    #[serde(flatten)]
    pub format: TableExtractFormat<S>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Compression {
    Deflate,
    Gzip,
    Snappy,
    Zstd,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "destinationFormat")]
pub enum TableExtractFormat<S = Box<str>> {
    Csv {
        #[serde(default, skip_serializing_if = "util::is_false")]
        print_header: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        field_delimiter: Option<S>,
    },
    NewlineDelimitedJson,
    Parquet,
    Avro {
        #[serde(
            default,
            rename = "useAvroLogicalTypes",
            skip_serializing_if = "util::is_false"
        )]
        use_avro_logical_types: bool,
    },
}
