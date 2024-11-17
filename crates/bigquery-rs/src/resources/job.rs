use std::collections::HashMap;

use super::ErrorProto;
use crate::{Error, util};

pub mod copy;
pub mod extract;
pub mod load;
pub mod query;

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

impl<S, Config> From<Config> for Job<S>
where
    JobConfiguration<S>: From<Config>,
{
    #[inline]
    fn from(value: Config) -> Self {
        JobConfiguration::from(value).into_job()
    }
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

impl JobStatus {
    pub fn into_result(mut self) -> Result<(), Error> {
        match (self.error_result, self.errors.len()) {
            (None, 0) => Ok(()),
            (Some(main), _) => Err(Error::JobError {
                main,
                misc: self.errors,
            }),
            (None, _) => {
                let main = self.errors.swap_remove(0);
                Err(Error::JobError {
                    main,
                    misc: self.errors,
                })
            }
        }
    }
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

    pub fn is_not_found(&self) -> bool
    where
        S: AsRef<str>,
    {
        if self
            .error_result
            .as_ref()
            .is_some_and(|err| err.is_not_found())
        {
            return true;
        }

        self.errors.iter().any(ErrorProto::is_not_found)
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
    pub labels: Option<HashMap<Box<str>, S>>,
    #[serde(flatten)]
    pub kind: JobConfigurationKind<S>,
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

impl<S, Kind> From<Kind> for JobConfiguration<S>
where
    JobConfigurationKind<S>: From<Kind>,
{
    #[inline]
    fn from(value: Kind) -> Self {
        let kind = JobConfigurationKind::from(value);

        JobConfiguration {
            job_type: Some(kind.job_type()),
            dry_run: false,
            job_timeout: None,
            labels: None,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationKind<S = Box<str>> {
    Query(query::JobConfigurationQuery<S>),
    Load(load::JobConfigurationLoad<S>),
    Copy(copy::JobConfigurationTableCopy<S>),
    Extract(extract::JobConfigurationExtract<S>),
}

impl<S> JobConfigurationKind<S> {
    pub fn job_type(&self) -> JobType {
        match self {
            Self::Copy(_) => JobType::Copy,
            Self::Extract(_) => JobType::Extract,
            Self::Load(_) => JobType::Load,
            Self::Query(_) => JobType::Query,
        }
    }
}

macro_rules! impl_from_job_config_kinds {
    ($($module_name:ident :: $name:ident -> $variant:ident),* $(,)?) => {
        $(
            impl<S> From<$module_name::$name<S>> for JobConfigurationKind<S> {
                #[inline]
                fn from(value: $module_name::$name<S>) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

impl_from_job_config_kinds! {
    query::JobConfigurationQuery -> Query,
    load::JobConfigurationLoad -> Load,
    copy::JobConfigurationTableCopy -> Copy,
    extract::JobConfigurationExtract -> Extract,
}
