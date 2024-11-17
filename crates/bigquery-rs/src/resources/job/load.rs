use std::collections::HashMap;

use crate::resources::TableReference;
use crate::resources::table::TableSchema;
use crate::util;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationLoad<S> {
    pub source_uris: Vec<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_set_spec_type: Option<FileSetSpecType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<TableSchema<S>>,
    pub destination_table: TableReference<S>,
    #[serde(
        // default = "Default::default",
        skip_serializing_if = "DestinationTableProperties::is_empty"
    )]
    pub destination_table_properties: DestinationTableProperties<S>,
    pub create_disposition: Option<CreateDisposition>,
    #[serde(default)]
    pub write_disposition: WriteDisposition,

    #[serde(flatten)]
    pub source_format: SourceFormat<S>,
    #[serde(skip_serializing_if = "util::is_false")]
    pub ignore_unknown_values: bool,
}

impl<S> JobConfigurationLoad<S> {
    pub fn new(
        source_uris: impl Into<Vec<S>>,
        source_format: impl Into<SourceFormat<S>>,
        destination_table: TableReference<S>,
    ) -> Self {
        Self {
            source_uris: source_uris.into(),
            file_set_spec_type: None,
            schema: None,
            destination_table,
            destination_table_properties: DestinationTableProperties::default(),
            create_disposition: None,
            write_disposition: WriteDisposition::default(),
            source_format: source_format.into(),
            ignore_unknown_values: false,
        }
    }

    pub fn write_truncate(mut self) -> Self {
        self.write_disposition = WriteDisposition::WriteTruncate;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkipLeadingRows {
    #[default]
    Autodetect,
    Skip(usize),
}

impl SkipLeadingRows {
    #[inline]
    pub const fn is_autodetect(&self) -> bool {
        matches!(self, Self::Autodetect)
    }
}

impl serde::Serialize for SkipLeadingRows {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Autodetect => serializer.serialize_none(),
            Self::Skip(rows) => serializer.serialize_some(&rows),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SkipLeadingRows {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<usize> as serde::Deserialize>::deserialize(deserializer)? {
            Some(rows) => Ok(Self::Skip(rows)),
            None => Ok(Self::Autodetect),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FileSetSpecType {
    /// This option expands source URIs by listing files from the object store.
    /// It is the default behavior if FileSetSpecType is not set.
    #[serde(rename = "FILE_SET_SPEC_TYPE_FILE_SYSTEM_MATCH")]
    FileSystemMatch,
    /// This option indicates that the provided URIs are newline-delimited manifest files,
    /// with one URI per line. Wildcard URIs are not supported.
    #[serde(rename = "FILE_SET_SPEC_TYPE_NEW_LINE_DELIMITED_MANIFEST")]
    NewLineDelimitedManifest,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTableProperties<S> {
    pub friendly_name: Option<S>,
    pub description: Option<S>,
    pub labels: Option<HashMap<Box<str>, S>>,
}

// cant use the derive(Default) impl since it places
// the unneeded bound S: Default
impl<S> Default for DestinationTableProperties<S> {
    fn default() -> Self {
        Self {
            friendly_name: None,
            description: None,
            labels: None,
        }
    }
}

impl<S> DestinationTableProperties<S> {
    pub fn is_empty(&self) -> bool {
        self.friendly_name.is_none()
            && self.description.is_none()
            && match self.labels {
                Some(ref labels) => labels.is_empty(),
                None => true,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreateDisposition {
    CreateIfNeeded,
    CreateNever,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WriteDisposition {
    #[default]
    WriteAppend,
    WriteTruncate,
    WriteEmpty,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Encoding {
    #[default]
    #[serde(rename = "UTF-8")]
    Utf8,
    #[serde(rename = "ISO-8859-1")]
    Iso8859_1,
    #[serde(rename = "UTF-16BE")]
    Utf16Be,
    #[serde(rename = "UTF-16LE")]
    Utf16Le,
    #[serde(rename = "UTF-32BE")]
    Utf32Be,
    #[serde(rename = "UTF-32LE")]
    Utf32Le,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "sourceFormat")]
pub enum SourceFormat<S> {
    Csv(CsvOptions<S>),
    DatastoreBackup,
    NewlineDelimitedJson {
        #[serde(rename = "jsonExtension", skip_serializing_if = "Option::is_none")]
        json_extension: Option<JsonExtension>,
        #[serde(skip_serializing_if = "util::is_false")]
        autodetect: bool,
    },
    Avro,
    Parquet,
    Orc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JsonExtension {
    #[serde(rename = "GEOJSON")]
    GeoJson,
}

impl<S> Default for SourceFormat<S> {
    fn default() -> Self {
        Self::Csv(CsvOptions::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvOptions<S> {
    #[serde(default, skip_serializing_if = "util::is_false")]
    pub allow_jagged_rows: bool,
    #[serde(default, skip_serializing_if = "SkipLeadingRows::is_autodetect")]
    pub skip_leading_rows: SkipLeadingRows,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub null_marker: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_delimiter: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bad_records: Option<usize>,
    #[serde(default, skip_serializing_if = "util::is_false")]
    pub autodetect: bool,
    #[serde(skip_serializing_if = "util::is_false")]
    pub allow_quoted_new_lines: bool,
}

impl<S> Default for CsvOptions<S> {
    fn default() -> Self {
        CsvOptions {
            allow_jagged_rows: false,
            skip_leading_rows: SkipLeadingRows::Autodetect,
            autodetect: true,
            null_marker: None,
            field_delimiter: None,
            quote: None,
            encoding: None,
            max_bad_records: None,
            allow_quoted_new_lines: false,
        }
    }
}
