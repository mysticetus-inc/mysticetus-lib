use std::sync::Arc;

use gcp_auth_channel::{Auth, Scope};
use http::header::HeaderValue;
use reqwest::{IntoUrl, Response, Url};

use super::dataset::DatasetClient;
use super::resources::job::Job;
use crate::Error;

#[allow(unused)]
const SCOPES: &[&str] = &["https://www.googleapis.com/auth/bigquery"];

/// The Base URL for this service, missing the project id (which is the next path component)
pub const BASE_URL: &str = "https://bigquery.googleapis.com/bigquery/v2/projects";

#[derive(Debug, Clone)]
pub struct BigQueryClient {
    pub(crate) inner: Arc<InnerClient>,
}

#[derive(Debug)]
pub(super) struct InnerClient {
    client: reqwest::Client,
    auth: Auth,
    base_url: Url,
}

impl BigQueryClient {
    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error> {
        let auth = Auth::new(project_id, scope).await?;

        let client = reqwest::Client::builder()
            .user_agent("bigquery-rs")
            .build()?;

        let mut base_url = Url::parse(BASE_URL).expect("base url is valid");

        base_url
            .path_segments_mut()
            .expect("can be a base")
            .push(project_id);

        Ok(Self {
            inner: Arc::new(InnerClient {
                client,
                auth,
                base_url,
            }),
        })
    }

    #[inline]
    pub fn project_id(&self) -> &str {
        self.inner.project_id()
    }

    pub async fn start_job(&self, job: Job) -> crate::Result<super::job::ActiveJob<'_>> {
        super::job::ActiveJob::new(self, job).await
    }

    pub fn dataset<D>(&self, dataset_name: D) -> DatasetClient<D> {
        DatasetClient::from_parts(dataset_name, Arc::clone(&self.inner))
    }
}

impl InnerClient {
    pub(crate) fn project_id(&self) -> &'static str {
        self.auth.project_id()
    }

    async fn get_auth_header(&self) -> Result<HeaderValue, Error> {
        self.auth.get_header().await.map_err(Error::from)
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub(crate) fn make_url<P>(&self, path: P) -> Url
    where
        P: IntoIterator,
        P::Item: AsRef<str>,
    {
        let mut new_url = self.base_url.clone();

        new_url
            .path_segments_mut()
            .expect("can be a base")
            .extend(path);

        new_url
    }

    #[inline]
    pub(crate) async fn request(
        &self,
        method: reqwest::Method,
        url: impl IntoUrl,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let header = self.get_auth_header().await?;

        let builder = self
            .client
            .request(method, url)
            .header(http::header::AUTHORIZATION, header);

        Ok(builder)
    }

    #[inline]
    pub(crate) async fn delete(&self, url: impl IntoUrl) -> Result<Response, Error> {
        self.request(reqwest::Method::DELETE, url)
            .await?
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }

    #[inline]
    pub(crate) async fn get(&self, url: impl IntoUrl) -> Result<Response, Error> {
        self.request(reqwest::Method::GET, url)
            .await?
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }

    #[inline]
    pub(crate) async fn post<S>(&self, url: impl IntoUrl, payload: S) -> Result<Response, Error>
    where
        S: serde::Serialize,
    {
        self.request(reqwest::Method::POST, url)
            .await?
            .json(&payload)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }
}
