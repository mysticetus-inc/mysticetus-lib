use std::ops::Range;
use std::sync::Arc;

use gcp_auth_channel::scopes::Scopes;
use gcp_auth_channel::{Auth, AuthManager, Scope};
use http::header::HeaderValue;
use reqwest::Response;

use super::bindings::BASE_URL;
use super::dataset::{DatasetClient, DatasetClientRef};
use crate::Error;

#[allow(unused)]
const SCOPES: &[&str] = &["https://www.googleapis.com/auth/bigquery"];

#[derive(Debug, Clone, PartialEq)]
pub struct BigQueryClient {
    inner: Arc<InnerClient>,
}

#[derive(Debug)]
pub(super) struct InnerClient {
    client: reqwest::Client,
    auth: Auth,
    base_url: Box<str>,
    /// the range within 'base_url' that contains the project id. This
    /// lets us avoid an extra string, and retains quick access (vs finding with
    /// "rsplit_once('/')")
    project_id_range: Range<usize>,
}

impl PartialEq for InnerClient {
    fn eq(&self, _rhs: &Self) -> bool {
        todo!()
    }
}

impl BigQueryClient {
    pub async fn new<S>(project_id: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let manager = AuthManager::new_shared().await?;
        let auth = Auth::new(manager);

        let client = reqwest::Client::builder()
            .user_agent("bigquery-rs")
            .build()?;

        let base_url = format!("{BASE_URL}/projects/{}", project_id.as_ref()).into_boxed_str();

        let project_id_range = (BASE_URL.len() + "/projects/".len())..base_url.len();

        Ok(Self {
            inner: Arc::new(InnerClient {
                client,
                auth,
                base_url,
                project_id_range,
            }),
        })
    }

    #[inline]
    pub fn project_id(&self) -> &str {
        self.inner.project_id()
    }

    pub fn dataset<D>(&self, dataset_name: D) -> DatasetClient<D> {
        DatasetClient::from_parts(dataset_name, Arc::clone(&self.inner))
    }

    pub fn dataset_ref<D>(&self, dataset_name: D) -> DatasetClientRef<'_, D> {
        DatasetClientRef::from_parts(dataset_name, &self.inner)
    }
}

impl InnerClient {
    pub(crate) fn project_id(&self) -> &str {
        // since Range isn't Copy, we need to re-construct it from the fields themselves.
        &self.base_url[self.project_id_range.start..self.project_id_range.end]
    }

    async fn get_auth_header(&self) -> Result<HeaderValue, Error> {
        let (_, token) = self
            .auth
            .get_header(Scopes::from_scope(Scope::BigQueryReadWrite))
            .await?;
        Ok(token)
    }

    pub(crate) fn base_url(&self) -> &str {
        &*self.base_url
    }

    pub(crate) async fn delete(&self, url: String) -> Result<Response, Error> {
        let header = self.get_auth_header().await?;
        self.client
            .delete(url)
            .header(http::header::AUTHORIZATION, header)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }

    pub(crate) async fn get(&self, url: String) -> Result<Response, Error> {
        let header = self.get_auth_header().await?;
        self.client
            .get(url)
            .header(http::header::AUTHORIZATION, header)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }

    pub(crate) async fn post<S>(&self, url: String, payload: S) -> Result<Response, Error>
    where
        S: serde::Serialize,
    {
        let header = self.get_auth_header().await?;
        self.client
            .post(url)
            .header(http::header::AUTHORIZATION, header)
            .json(&payload)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::from)
    }
}
