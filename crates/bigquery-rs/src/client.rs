use std::sync::Arc;

use bigquery_resources_rs::ErrorProto;
use bigquery_resources_rs::query::{QueryRequest, QueryString};
use gcp_auth_channel::{Auth, Scope};
use http::header::HeaderValue;
use reqwest::{IntoUrl, Response, Url};

use super::dataset::DatasetClient;
use crate::Error;
use crate::query::QueryBuilder;
use crate::resources::job::Job;

/// The Base URL for this service, missing the project id (which is the next path component)
pub(crate) const BASE_URL: &str = "https://bigquery.googleapis.com/bigquery/v2/projects";

#[derive(Debug, Clone)]
pub struct BigQueryClient {
    pub(crate) inner: Arc<InnerClient>,
}

#[derive(Debug)]
pub(crate) struct InnerClient {
    client: reqwest::Client,
    auth: Auth,
    base_url: Url,
}

impl BigQueryClient {
    pub fn new_from_parts(auth: Auth, client: reqwest::Client) -> Self {
        let mut base_url = Url::parse(BASE_URL).expect("base url is valid");

        base_url
            .path_segments_mut()
            .expect("can be a base")
            .push(auth.project_id());

        Self {
            inner: Arc::new(InnerClient {
                client,
                auth,
                base_url,
            }),
        }
    }

    pub fn new_from_auth(auth: Auth) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .user_agent("bigquery-rs")
            .build()?;

        Ok(Self::new_from_parts(auth, client))
    }

    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error> {
        let auth = Auth::new(project_id, scope).await?;
        Self::new_from_auth(auth)
    }

    #[inline]
    pub fn project_id(&self) -> &'static str {
        self.inner.project_id()
    }

    pub async fn start_job<S>(&self, job: Job<S>) -> crate::Result<super::job::ActiveJob<'_>>
    where
        Job<S>: serde::Serialize,
    {
        super::job::ActiveJob::new(self, job).await
    }

    pub fn query<S>(&self, query: QueryString) -> QueryBuilder<S> {
        QueryBuilder::new(self.clone(), QueryRequest::new(query))
    }

    pub fn dataset<D>(&self, dataset_name: D) -> DatasetClient<'_, D> {
        DatasetClient::from_parts(dataset_name, &self.inner)
    }

    pub fn into_dataset<D>(self, dataset_name: D) -> DatasetClient<'static, D> {
        DatasetClient::from_parts(dataset_name, self.inner)
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
        let resp = self
            .request(reqwest::Method::DELETE, url)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            Err(handle_error(resp).await)
        } else {
            Ok(resp)
        }
    }

    #[inline]
    pub(crate) async fn get(&self, url: impl IntoUrl) -> Result<Response, Error> {
        let resp = self
            .request(reqwest::Method::GET, url)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            Err(handle_error(resp).await)
        } else {
            Ok(resp)
        }
    }

    #[inline]
    pub(crate) async fn post<S>(&self, url: impl IntoUrl, payload: S) -> Result<Response, Error>
    where
        S: serde::Serialize,
    {
        let resp = self
            .request(reqwest::Method::POST, url)
            .await?
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            Err(handle_error(resp).await)
        } else {
            Ok(resp)
        }
    }
}

pub(crate) async fn handle_error(response: reqwest::Response) -> crate::Error {
    let status = response.status();
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return error.into(),
    };

    fn try_deserialize_json_error(
        text: &str,
        status: u16,
    ) -> Result<Option<crate::Error>, path_aware_serde::Error<serde_json::Error>> {
        fn handle_error_proto_array(mut array: Vec<ErrorProto>, status: u16) -> crate::Error {
            match array.len() {
                0 => ErrorProto::new("no error information given".into())
                    .with_status(status)
                    .into(),
                1.. => {
                    let main = array.remove(0);
                    crate::Error::JobError { main, misc: array }
                }
            }
        }

        if text.starts_with('[') {
            let errors: Vec<ErrorProto> = path_aware_serde::json::deserialize_str(text)?;
            Ok(Some(handle_error_proto_array(errors, status)))
        } else if text.starts_with('{') {
            // check for a nested object with an errors array,
            // within the first 5 chars. That should handle
            // whitespace between the opening brace and the key
            //
            // i.e '{\s+"errors":[...]}'

            let stop_looking_after = text.ceil_char_boundary(5 + "\"errors\":".len());
            match text[..stop_looking_after].find("\"errors\":") {
                Some(_) => {
                    #[derive(serde::Deserialize)]
                    struct Errors {
                        errors: Vec<ErrorProto>,
                    }

                    let Errors { errors } = path_aware_serde::json::deserialize_str(text)?;

                    Ok(Some(handle_error_proto_array(errors, status)))
                }
                None => {
                    let main = path_aware_serde::json::deserialize_str::<ErrorProto>(text)?
                        .with_status(status);
                    Ok(Some(crate::Error::from(main)))
                }
            }
        } else {
            Ok(None)
        }
    }

    match try_deserialize_json_error(&text, status.as_u16()) {
        Ok(Some(error)) => return error,
        // this is if the response is text based and not json
        Ok(None) => (),
        // if we failed to deserialize the json, log it
        Err(error) => tracing::warn!(
            message = "failed to deserialize error json, falling back to raw text",
            ?error
        ),
    }

    ErrorProto::new(text.into_boxed_str())
        .with_status(status.as_u16())
        .into()
}

pub(crate) async fn handle_json_response<T>(response: reqwest::Response) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    if !response.status().is_success() {
        return Err(handle_error(response).await);
    }

    deserialize_json(response).await
}

pub(crate) async fn deserialize_json<T>(response: reqwest::Response) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = response.bytes().await?;
    path_aware_serde::json::deserialize_slice(&bytes).map_err(crate::Error::from)
}
