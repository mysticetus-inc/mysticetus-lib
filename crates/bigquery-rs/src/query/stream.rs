use std::pin::Pin;
use std::task::{Context, Poll};

use bigquery_resources_rs::job::JobReference;
use bigquery_resources_rs::query::{QueryRequest, QueryResponse};
use futures::Stream;
use reqwest::RequestBuilder;
use timestamp::Duration;
use tokio_util::sync::ReusableBoxFuture;

use crate::{BigQueryClient, MissingField};

pin_project_lite::pin_project! {
    pub struct QueryStream<Row, S: AsRef<str> = Box<str>> {
        client: BigQueryClient,
        #[pin]
        fut: ReusableBoxFuture<'static, crate::Result<QueryResponse<Row, S>>>,
        done: bool,
        options: Options,
        job_reference: Option<JobReference<S>>,
        offset: u64,
    }
}

impl<Row, S> QueryStream<Row, S>
where
    Row: serde::de::DeserializeOwned + 'static,
    S: AsRef<str> + serde::de::DeserializeOwned + serde::Serialize + Clone + Send + Sync + 'static,
    QueryResponse<Row, S>: std::fmt::Debug,
{
    pub(super) fn new(client: BigQueryClient, options: Options, request: QueryRequest<S>) -> Self {
        let fut = ReusableBoxFuture::new(call::<Row, S>(
            client.clone(),
            RequestType::Initial(request),
        ));

        Self {
            client,
            fut,
            options,
            done: false,
            job_reference: None,
            offset: 0,
        }
    }
}

impl<Row, S> Stream for QueryStream<Row, S>
where
    Row: serde::de::DeserializeOwned + 'static,
    S: AsRef<str> + serde::de::DeserializeOwned + serde::Serialize + Clone + Send + Sync + 'static,
    QueryResponse<Row, S>: std::fmt::Debug,
{
    type Item = crate::Result<QueryResponse<Row, S>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if *this.done {
            return Poll::Ready(None);
        }

        let result = std::task::ready!(this.fut.as_mut().poll(cx));

        *this.done = match result {
            Ok(ref res) => res.results.job_complete,
            Err(_) => true,
        };

        let mut response = result?;

        *this.offset += response.results.rows.len() as u64;

        if !response.results.job_complete {
            let job_ref = match *this.job_reference {
                Some(ref job_ref) => job_ref,
                None => match response.job_reference {
                    Some(ref job_ref) => this.job_reference.insert(job_ref.clone()),
                    None => {
                        *this.done = true;
                        return Poll::Ready(Some(Err(crate::Error::MissingField(
                            MissingField::new_missing::<JobReference<S>>("job_reference"),
                        ))));
                    }
                },
            };

            this.fut.get_mut().set(call::<Row, S>(
                this.client.clone(),
                RequestType::Successive {
                    job_id: job_ref.job_id.clone(),
                    start_index: Some(*this.offset),
                    page_token: response.results.page_token.take(),
                    options: this.options.clone(),
                },
            ));
        }

        Poll::Ready(Some(Ok(response)))
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub per_request_timeout: Option<Duration>,
    pub location: Option<Box<str>>,
    pub max_results: Option<u64>,
}

impl Options {
    fn insert_query_params(&self, mut builder: RequestBuilder) -> RequestBuilder {
        if let Some(timeout) = self.per_request_timeout {
            builder = builder.query(&[("timeoutMs", timeout.millis())]);
        }

        if let Some(ref location) = self.location {
            builder = builder.query(&[("location", location)]);
        }

        if let Some(max_results) = self.max_results {
            builder = builder.query(&[("maxResults", max_results)]);
        }

        builder
    }
}

enum RequestType<S> {
    Initial(QueryRequest<S>),
    Successive {
        job_id: S,
        start_index: Option<u64>,
        page_token: Option<S>,
        options: Options,
    },
}

async fn call<Row, S>(
    client: BigQueryClient,
    request: RequestType<S>,
) -> crate::Result<QueryResponse<Row, S>>
where
    Row: serde::de::DeserializeOwned,
    S: AsRef<str> + serde::de::DeserializeOwned + serde::Serialize,
    QueryResponse<Row, S>: std::fmt::Debug,
{
    match request {
        RequestType::Initial(req) => super::call_query(&client, &req).await,
        RequestType::Successive {
            job_id,
            start_index,
            page_token,
            options,
        } => call_successive(client, job_id, start_index, page_token, options).await,
    }
}

async fn call_successive<Row, S>(
    client: BigQueryClient,
    job_id: S,
    start_index: Option<u64>,
    page_token: Option<S>,
    options: Options,
) -> crate::Result<QueryResponse<Row, S>>
where
    Row: serde::de::DeserializeOwned,
    S: AsRef<str> + serde::de::DeserializeOwned + serde::Serialize,
{
    let url = client.inner.make_url(["queries", job_id.as_ref()]);

    let mut builder = client.inner.request(reqwest::Method::GET, url).await?;

    if let Some(start_index) = start_index {
        builder = builder.query(&[("startIndex", start_index)]);
    }

    if let Some(ref page_token) = page_token {
        builder = builder.query(&[("pageToken", page_token.as_ref())]);
    }

    let resp = options.insert_query_params(builder).send().await?;

    crate::client::handle_json_response(resp).await
}
