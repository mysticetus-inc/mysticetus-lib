use std::borrow::Cow;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::time::Interval;
use tokio_util::sync::ReusableBoxFuture;

use crate::resources::job::{Job, JobReference, JobState, JobStatus};

const DEFAULT_POLL_FREQUENCY: timestamp::Duration = timestamp::Duration::from_seconds(2);

pub struct ActiveJob<'a> {
    client: Cow<'a, super::BigQueryClient>,
    job: Job,
    job_ref: JobReference,
    status: JobStatus,
}

impl<'a> ActiveJob<'a> {
    pub(crate) async fn new<S>(
        client: &'a super::BigQueryClient,
        job: Job<S>,
    ) -> crate::Result<Self>
    where
        Job<S>: serde::Serialize,
    {
        let url = client.inner.make_url(&["jobs"]);
        let resp = client.inner.post(url, &job).await?;

        let mut job: Job = super::client::deserialize_json(resp).await?;

        let status = job
            .status
            .take()
            .ok_or_else(|| crate::Error::missing_field::<Job>("status", None::<JobStatus>))?;

        let job_ref = job.job_reference.take().ok_or_else(|| {
            crate::Error::missing_field::<Job>("job_reference", None::<JobReference>)
        })?;

        Ok(ActiveJob {
            client: Cow::Borrowed(client),
            job,
            status,
            job_ref,
        })
    }

    pub fn into_owned(self) -> ActiveJob<'static> {
        ActiveJob {
            client: Cow::Owned(self.client.into_owned()),
            job: self.job,
            job_ref: self.job_ref,
            status: self.status,
        }
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn poll_with_callback<Cb>(
        self,
        frequency: timestamp::Duration,
        callback: Cb,
    ) -> ActiveJobFuture<'a, Cb> {
        let mut url = self.client.inner.make_url(&["jobs", &self.job_ref.job_id]);

        url.query_pairs_mut()
            .append_pair("location", &self.job_ref.location)
            .finish();

        let client = (&*self.client).clone();

        ActiveJobFuture {
            request_fut: ReusableBoxFuture::new(get_job(url.clone(), client)),
            url,
            poll_interval: tokio::time::interval(frequency.into()),
            state: State::Requesting,
            callback,
            job: self,
        }
    }

    pub fn poll(self, frequency: timestamp::Duration) -> ActiveJobFuture<'a> {
        self.poll_with_callback(frequency, noop_callback)
    }
}

async fn get_job(url: reqwest::Url, client: super::BigQueryClient) -> crate::Result<Job> {
    let resp = client.inner.get(url).await?;
    super::client::deserialize_json(resp).await
}

impl<'a> IntoFuture for ActiveJob<'a> {
    type Output = crate::Result<JobStatus>;
    type IntoFuture = ActiveJobFuture<'a>;

    fn into_future(self) -> Self::IntoFuture {
        self.poll(DEFAULT_POLL_FREQUENCY)
    }
}

#[inline]
fn noop_callback(_: JobState) -> Result<(), std::convert::Infallible> {
    Ok(())
}

pin_project_lite::pin_project! {
    pub struct ActiveJobFuture<'a, Cb = fn(JobState) -> Result<(), std::convert::Infallible>> {
        job: ActiveJob<'a>,
        url: reqwest::Url,
        request_fut: ReusableBoxFuture<'a, crate::Result<Job>>,
        #[pin]
        poll_interval: Interval,
        state: State,
        callback: Cb,
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Done,
    Requesting,
    Waiting,
}

impl<Cb, E> Future for ActiveJobFuture<'_, Cb>
where
    Cb: FnMut(JobState) -> Result<(), E>,
    crate::Error: From<E>,
{
    type Output = crate::Result<JobStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match *this.state {
                State::Done => return Poll::Ready(Ok(this.job.status.take())),
                State::Requesting => {
                    this.job.job = std::task::ready!(this.request_fut.poll(cx))?;

                    this.job.status = this.job.job.status.take().ok_or_else(|| {
                        crate::Error::missing_field::<Job>("status", None::<JobStatus>)
                    })?;

                    (this.callback)(this.job.status.state)?;

                    if this.job.status.state == JobState::Done {
                        *this.state = State::Done;
                    } else {
                        // reset so we start the interval after a request finishes
                        this.poll_interval.reset();
                        *this.state = State::Waiting;
                    }
                }
                State::Waiting => {
                    std::task::ready!(this.poll_interval.as_mut().poll_tick(cx));
                    // kick off a new request once this interval timer ticks
                    let url = this.url.clone();
                    let client = (&*this.job.client).clone();

                    *this.state = State::Requesting;
                    this.request_fut.set(get_job(url, client));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::TableReference;
    use crate::resources::job::extract::*;
    use crate::resources::job::*;

    fn build_test_job() -> Job {
        let mut job = Job::from(JobConfigurationExtract {
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
        });

        job.configuration.job_timeout = Some(timestamp::Duration::from_seconds(180));

        job
    }

    #[tokio::test]
    async fn test_job_export() -> crate::Result<()> {
        let client = super::super::BigQueryClient::new(
            "mysticetus-boem",
            gcp_auth_channel::Scope::BigQueryAdmin,
        )
        .await?;

        let job = build_test_job();

        let active_job = client.start_job(job).await?;

        let freq = timestamp::Duration::from_seconds(10);

        let log_status = |status| {
            println!("{status:#?}");
            Ok(()) as Result<(), std::convert::Infallible>
        };

        active_job.poll_with_callback(freq, log_status).await?;

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

        let job: Job = path_aware_serde::json::deserialize_str(JSON_JOB).unwrap();
        println!("{job:#?}");
    }
}
