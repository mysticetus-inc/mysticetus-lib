use protos::bigquery_v2::ModelReference;

use super::bindings::JobReference;

pub struct JobClient {
    client: super::BigQueryClient,
}

impl JobClient {
    pub(crate) fn new(client: super::BigQueryClient) -> Self {
        Self { client }
    }

    pub async fn start_job(&self, job: Job) -> Result<ActiveJob, crate::Error> {
        let url = format!("{}/jobs", self.client.inner.base_url());
        let resp = self.client.inner.post(url, &job).await?;

        let mut job: Job = resp.json().await?;

        let status = job
            .status
            .take()
            .ok_or_else(|| crate::Error::Misc(format!("job status not returned by Google")))?;

        let job_ref = job
            .job_reference
            .take()
            .ok_or_else(|| crate::Error::Misc(format!("job id not returned by Google")))?;

        Ok(ActiveJob {
            client: self.client.clone(),
            job,
            status,
            poll_url: None,
            job_ref,
        })
    }
}

pub struct ActiveJob {
    client: super::BigQueryClient,
    job: Job,
    job_ref: JobReference,
    poll_url: Option<String>,
    status: JobStatus,
}

impl ActiveJob {
    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    fn get_or_insert_poll_url(&mut self) -> String {
        match self.poll_url {
            Some(ref url) => url.clone(),
            None => self
                .poll_url
                .insert(format!(
                    "{}/jobs/{}?location={}",
                    self.client.inner.base_url(),
                    self.job_ref.job_id,
                    self.job_ref.location
                ))
                .clone(),
        }
    }

    pub async fn poll_until_done<Cb>(
        &mut self,
        frequency: timestamp::Duration,
        mut callback: Cb,
    ) -> crate::Result<()>
    where
        Cb: FnMut(JobState) -> Result<(), crate::Error>,
    {
        let mut interval = tokio::time::interval(frequency.into());

        while !matches!(self.status.state, JobState::Done) {
            let status = self.poll_job().await?;
            callback(status)?;

            if status == JobState::Done {
                break;
            }

            interval.tick().await;
        }

        Ok(())
    }

    pub async fn poll_job(&mut self) -> Result<JobState, crate::Error> {
        if matches!(self.status.state, JobState::Done) {
            return Ok(self.status.state);
        }

        let url = self.get_or_insert_poll_url();
        self.job = self.client.inner.get(url).await?.json().await?;

        self.status = self
            .job
            .status
            .take()
            .ok_or_else(|| crate::Error::Misc(format!("job status not returned by Google")))?;

        Ok(self.status.state)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    self_link: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_email: Option<Box<str>>,
    configuration: JobConfiguration,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_reference: Option<JobReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statistics: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<JobStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    principal_subject: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    error_result: Option<ErrorProto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    errors: Vec<ErrorProto>,
    state: JobState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobState {
    Pending,
    Running,
    Done,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    reason: Box<str>,
    location: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug_info: Option<Box<str>>,
    message: Box<str>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    job_type: Option<JobType>,
    #[serde(default, skip_serializing_if = "is_false")]
    dry_run: bool,
    #[serde(
        default,
        rename = "jobTimeoutMs",
        with = "crate::rest::util::timeout_ms::optional",
        skip_serializing_if = "Option::is_none"
    )]
    job_timeout: Option<timestamp::Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<std::collections::HashMap<Box<str>, Box<str>>>,
    #[serde(flatten)]
    kind: JobConfigurationKind,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationKind {
    Query(JobConfigurationQuery),
    Load(JobConfigurationLoad),
    Copy(JobConfigurationTableCopy),
    Extract(JobConfigurationExtract),
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
pub struct JobConfigurationExtract {
    destination_uris: Vec<Box<str>>,
    #[serde(flatten)]
    kind: ExtractKind,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ExtractKind {
    Table(TableExtract),
    // TODO: Model
    // Model,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableExtract {
    source_table: TableReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    compression: Option<Compression>,
    #[serde(flatten)]
    format: TableExtractFormat,
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
pub enum TableExtractFormat {
    Csv {
        #[serde(default, skip_serializing_if = "is_false")]
        print_header: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field_delimiter: Option<Box<str>>,
    },
    NewlineDelimitedJson,
    Parquet,
    Avro {
        #[serde(
            default,
            rename = "useAvroLogicalTypes",
            skip_serializing_if = "is_false"
        )]
        use_avro_logical_types: bool,
    },
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableReference {
    project_id: Box<str>,
    dataset_id: Box<str>,
    table_id: Box<str>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_job() -> Job {
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
            configuration: JobConfiguration {
                job_type: None,
                dry_run: false,
                job_timeout: Some(timestamp::Duration::from_seconds(180)),
                labels: None,
                kind: JobConfigurationKind::Extract(JobConfigurationExtract {
                    destination_uris: vec!["gs://boem-backups/effort-test".into()],
                    kind: ExtractKind::Table(TableExtract {
                        source_table: TableReference {
                            project_id: "mysticetus-boem".into(),
                            dataset_id: "main".into(),
                            table_id: "effort".into(),
                        },
                        compression: Some(Compression::Gzip),
                        format: TableExtractFormat::Csv {
                            print_header: true,
                            field_delimiter: None,
                        },
                    }),
                }),
            },
        }
    }

    #[tokio::test]
    async fn test_job_export() -> crate::Result<()> {
        let client = super::super::BigQueryClient::new(
            "mysticetus-boem",
            gcp_auth_channel::Scope::BigQueryAdmin,
        )
        .await?
        .into_job_client();

        let job = build_test_job();

        let mut active_job = client.start_job(job).await?;

        let freq = timestamp::Duration::from_seconds(10);

        let log_status = |status| {
            println!("{status:#?}");
            Ok(())
        };

        active_job.poll_until_done(freq, log_status).await?;

        Ok(())
    }

    #[test]
    fn test_job_json_serialize() {
        let job = build_test_job();

        let serialized_job = serde_json::to_string_pretty(&job).unwrap();

        let deserialized_job: Job = serde_json::from_str(&serialized_job).unwrap();

        assert_eq!(deserialized_job, job);
    }

    #[test]
    fn test_job_json_deserialize() {
        const JSON_JOB: &str = r#"{
          "configuration": {
            "extract": {
              "compression": "GZIP",
              "destinationFormat": "CSV",
              "destinationUri": "gs://boem-backups/effort-test",
              "destinationUris": [
                "gs://boem-backups/effort-test"
              ],
              "printHeader": true,
              "sourceTable": {
                "datasetId": "main",
                "projectId": "mysticetus-boem",
                "tableId": "effort"
              }
            },
            "jobTimeoutMs": "180000",
            "jobType": "EXTRACT"
          },
          "etag": "R44Ak0MU41Ssxc+JHAtWTg==",
          "id": "mysticetus-boem:us-central1.job_CSYzdJ_xqkeeOXIflmREwW3OSE0m",
          "jobCreationReason": {
            "code": "REQUESTED"
          },
          "jobReference": {
            "jobId": "job_CSYzdJ_xqkeeOXIflmREwW3OSE0m",
            "location": "us-central1",
            "projectId": "mysticetus-boem"
          },
          "kind": "bigquery#job",
          "principal_subject": "user:mrudisel@mysticetus.com",
          "selfLink": "https://www.googleapis.com/bigquery/v2/projects/mysticetus-boem/jobs/job_CSYzdJ_xqkeeOXIflmREwW3OSE0m?location=us-central1",
          "statistics": {
            "creationTime": "1731550230831",
            "reservation_id": "default-pipeline",
            "startTime": "1731550230926"
          },
          "status": {
            "state": "RUNNING"
          },
          "user_email": "mrudisel@mysticetus.com"
        }"#;

        let job: Job =
            path_aware_serde::deserialize_json(serde_json::de::StrRead::new(JSON_JOB)).unwrap();
        println!("{job:#?}");
    }
}
