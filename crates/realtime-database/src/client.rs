use std::fmt;
use std::sync::Arc;

use gcp_auth_provider::Auth;
use reqwest::Response;
use reqwest::header::{self, HeaderValue};

// use parking_lot::RwLock;
use crate::error::{Error, RealtimeDbError};
use crate::event::EventStream;
use crate::path::RtDbPath;

const EVENT_STREAM_VALUE: HeaderValue = HeaderValue::from_static("text/event-stream");

const TYPED_NONE: Option<&()> = None;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpMethod {
    Delete,
    Get,
    Patch,
    Post,
    Put,
}

impl HttpMethod {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delete => "DELETE",
            Self::Get => "GET",
            Self::Patch => "PATCH",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }

    pub const fn supports_silent_print(&self) -> bool {
        matches!(self, Self::Patch | Self::Post | Self::Put)
    }

    pub const fn supports_shallow(&self) -> bool {
        matches!(self, Self::Get)
    }

    #[allow(dead_code)] // TODO: work on etag support
    pub const fn supports_etag_header(&self) -> bool {
        !matches!(self, Self::Patch)
    }
}

#[derive(Clone)]
pub struct Client {
    /// The base database URL.
    pub(crate) db_url: Arc<str>,
    /// Handles tokens (as well as caching them)
    pub(crate) auth: Auth,
    /// The Http Client.
    pub(crate) client: reqwest::Client,
    /// Whether to include the silent print query flag, which tells the server to
    /// ignore sending written back, improving performance.
    silent_print: bool,
}

impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt("Client", formatter)
    }
}

impl Client {
    pub(crate) fn new(
        db_url: Arc<str>,
        auth: Auth,
        client: reqwest::Client,
        silent_print: bool,
    ) -> Self {
        Self {
            db_url,
            auth,
            client,
            silent_print,
        }
    }

    pub(crate) fn debug_fmt(
        &self,
        name: &'static str,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        formatter
            .debug_struct(name)
            .field("db_url", &self.db_url)
            .field("auth", &self.auth)
            .field("client", &self.client)
            .field("silent_print", &self.silent_print)
            .finish()
    }

    async fn get_auth_header(&self) -> Result<HeaderValue, Error> {
        match self.auth.get_header() {
            gcp_auth_provider::GetHeaderResult::Cached(cached) => Ok(cached.header),
            gcp_auth_provider::GetHeaderResult::Refreshing(fut) => Ok(fut.await?.header),
        }
    }

    /// Builds the base request. This includes building the URL, inserting the auth header value,
    /// and including the silent_print query param if configured.
    async fn build_base_request<Q, P>(
        &self,
        method: HttpMethod,
        path: &P,
        query: Option<&Q>,
        shallow: Option<bool>,
    ) -> Result<reqwest::RequestBuilder, Error>
    where
        P: RtDbPath,
        Q: serde::Serialize,
    {
        let auth_header = self.get_auth_header().await?;

        let mut url: String = (&*self.db_url).to_owned();
        path.complete_base_url(&mut url);

        let mut request_builder = match method {
            HttpMethod::Delete => self.client.delete(url),
            HttpMethod::Get => self.client.get(url),
            HttpMethod::Patch => self.client.patch(url),
            HttpMethod::Post => self.client.post(url),
            HttpMethod::Put => self.client.put(url),
        };

        // add the auth header
        request_builder = request_builder.header(header::AUTHORIZATION, auth_header);

        if let Some(query) = query {
            request_builder = request_builder.query(query);
        }

        // check for the silent print flag.
        if method.supports_silent_print() && self.silent_print {
            request_builder = request_builder.query(&[("print", "silent")]);
        }

        if method.supports_shallow() && shallow.unwrap_or(false) {
            request_builder = request_builder.query(&[("shallow", "true")]);
        }

        Ok(request_builder)
    }

    pub(crate) async fn get<P>(&self, path: &P, shallow: bool) -> Result<Response, Error>
    where
        P: RtDbPath,
    {
        let response = self
            .build_base_request(HttpMethod::Get, path, TYPED_NONE, Some(shallow))
            .await?
            .send()
            .await?;

        handle_response_errors(response).await
    }

    pub(crate) async fn start_event_stream<P>(
        &self,
        path: &P,
        shallow: bool,
    ) -> Result<EventStream, Error>
    where
        P: RtDbPath,
    {
        let response = self
            .build_base_request(HttpMethod::Get, path, TYPED_NONE, Some(shallow))
            .await?
            .header(header::ACCEPT, EVENT_STREAM_VALUE.clone())
            .send()
            .await?
            .error_for_status()?;

        Ok(EventStream::from_response(response))
    }

    pub(crate) async fn get_with_query<P, Q>(
        &self,
        path: &P,
        query: &Q,
        shallow: bool,
    ) -> Result<Response, Error>
    where
        P: RtDbPath,
        Q: serde::Serialize,
    {
        let response = self
            .build_base_request(HttpMethod::Get, path, Some(query), Some(shallow))
            .await?
            .send()
            .await?;

        handle_response_errors(response).await
    }

    pub(crate) async fn put<P, T>(&self, path: &P, body: &T) -> Result<Response, Error>
    where
        P: RtDbPath,
        T: serde::Serialize,
    {
        let response = self
            .build_base_request(HttpMethod::Put, path, TYPED_NONE, None)
            .await?
            .json(body)
            .send()
            .await?;

        handle_response_errors(response).await
    }

    pub(crate) async fn patch<P, T>(&self, path: &P, body: &T) -> Result<Response, Error>
    where
        P: RtDbPath,
        T: serde::Serialize,
    {
        let response = self
            .build_base_request(HttpMethod::Patch, path, TYPED_NONE, None)
            .await?
            .json(body)
            .send()
            .await?;

        handle_response_errors(response).await
    }

    pub(crate) async fn post<P, T>(&self, path: &P, body: &T) -> Result<Response, Error>
    where
        P: RtDbPath,
        T: serde::Serialize,
    {
        let response = self
            .build_base_request(HttpMethod::Post, path, TYPED_NONE, None)
            .await?
            .json(body)
            .send()
            .await?;

        handle_response_errors(response).await
    }

    pub(crate) async fn delete<P>(&self, path: &P) -> Result<Response, Error>
    where
        P: RtDbPath,
    {
        let response = self
            .build_base_request(HttpMethod::Delete, path, TYPED_NONE, None)
            .await?
            .send()
            .await?;

        handle_response_errors(response).await
    }
}

async fn handle_response_errors(response: Response) -> Result<Response, Error> {
    match response.error_for_status_ref() {
        Ok(_) => Ok(response),
        Err(reqwest_err) => match response.json::<RealtimeDbError>().await {
            Ok(base_error) => Err(base_error.with_reqwest_error(reqwest_err).into()),
            Err(_) => Err(reqwest_err.into()),
        },
    }
}
