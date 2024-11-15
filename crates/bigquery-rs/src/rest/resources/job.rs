use super::{ErrorProto, TableReference};
use crate::rest::util;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job<S = Box<str>> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<S>,
    pub configuration: JobConfiguration<S>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_reference: Option<JobReference<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<JobStatus<S>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_subject: Option<S>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobReference<S = Box<str>> {
    pub job_id: S,
    /// The geographic location of the job. See details at
    /// https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    pub location: S,
    /// [Required] The ID of the project containing this job.
    pub project_id: S,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus<S = Box<str>> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_result: Option<ErrorProto<S>>,
    // need to specify a default fn vec to avoid S needing Default
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorProto<S>>,
    pub state: JobState,
}

impl<S> JobStatus<S> {
    pub fn len(&self) -> usize {
        self.error_result.is_some() as usize + self.errors.len()
    }

    pub fn first_error(&self) -> Option<&ErrorProto<S>> {
        self.error_result.as_ref().or_else(|| self.errors.first())
    }

    pub fn into_errors(mut self) -> Vec<ErrorProto<S>> {
        if let Some(result) = self.error_result {
            self.errors.insert(0, result);
        }

        self.errors
    }
    
    pub(crate) fn take(&mut self) -> Self {
        Self {
            errors: std::mem::take(&mut self.errors),
            error_result: self.error_result.take(),
            state: self.state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobState {
    Pending,
    Running,
    Done,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobType {
    Query,
    Load,
    Extract,
    Copy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration<S> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_type: Option<JobType>,
    #[serde(default, skip_serializing_if = "util::is_false")]
    pub dry_run: bool,
    #[serde(
        default,
        rename = "jobTimeoutMs",
        with = "util::duration_ms::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub job_timeout: Option<timestamp::Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<Box<str>, S>>,
    #[serde(flatten)]
    pub kind: JobConfigurationKind,
}

impl<S> JobConfiguration<S> {
    /// Converts [self] into a [Job], with empty values for the fields in [Job]
    pub fn into_job(self) -> Job<S> {
        Job {
            kind: None,
            etag: None,
            id: None,
            self_link: None,
            user_email: None,
            job_reference: None,
            statistics: None,
            status: None,
            principal_subject: None,
            configuration: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationKind<S = Box<str>> {
    Query(JobConfigurationQuery),
    Load(JobConfigurationLoad),
    Copy(JobConfigurationTableCopy),
    Extract(JobConfigurationExtract<S>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationQuery {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationLoad {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationTableCopy {}

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
    // TODO: Model
    // Model,
}

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
