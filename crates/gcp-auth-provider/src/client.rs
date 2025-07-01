use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http::HeaderValue;
use http_body::{Body, Frame};
use hyper::body::Incoming;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::client::legacy::connect::dns::GaiResolver;
use hyper_util::client::legacy::connect::{Connect, HttpConnector};
use hyper_util::rt;
use tower::Service;

use crate::Error;

const USER_AGENT: HeaderValue =
    HeaderValue::from_static(concat!("gcp-auth-provider ", env!("CARGO_PKG_VERSION")));

/// Simple HTTP 1|2 client that avoids the overhead of using reqwest.
///
/// TODO: replace with a client based on [hyper::client], instead of
/// [hyper_util::client] since the latter does fairly complex connection
/// pooling that we wont need (we'll only be making requests to 1 host
/// in practice)
#[derive(Debug, Clone)]
pub(crate) struct Client<Connector> {
    client: HyperClient<Connector, BytesBody>,
}

pub(crate) type HttpClient<R = GaiResolver> = Client<HttpConnector<R>>;
pub(crate) type HttpsClient<R = GaiResolver> = Client<HttpsConnector<HttpConnector<R>>>;

fn make_https_connection<R>(resolver: R) -> std::io::Result<HttpsConnector<HttpConnector<R>>> {
    let builder = {
        #[cfg(feature = "webpki-roots")]
        {
            hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots()
        }
        #[cfg(not(feature = "webpki-roots"))]
        {
            hyper_rustls::HttpsConnectorBuilder::new().with_native_roots()?
        }
    };

    let mut connector = HttpConnector::new_with_resolver(resolver);
    connector.enforce_http(false);
    Ok(builder
        .https_only()
        .enable_all_versions()
        .wrap_connector(connector))
}

impl Client<HttpConnector<GaiResolver>> {
    #[inline]
    pub(crate) fn new_http() -> Self {
        Self::new_from_connector(HttpConnector::new())
    }
}

impl Client<HttpsConnector<HttpConnector<GaiResolver>>> {
    #[inline]
    pub(crate) fn new_https() -> std::io::Result<Self> {
        make_https_connection(GaiResolver::new()).map(Self::new_from_connector)
    }
}

fn insert_headers_for_request(request: &mut http::Request<BytesBody>) {
    fn insert_missing_header(
        headers: &mut http::HeaderMap,
        name: http::HeaderName,
        make_header: impl FnOnce() -> HeaderValue,
    ) {
        if let http::header::Entry::Vacant(vacant) = headers.entry(name) {
            vacant.insert(make_header());
        }
    }
    let body_len = request.body().len();
    let headers = request.headers_mut();

    insert_missing_header(headers, http::header::ACCEPT, || {
        HeaderValue::from_static("*/*")
    });

    insert_missing_header(headers, http::header::USER_AGENT, || USER_AGENT);

    insert_missing_header(headers, http::header::CONTENT_LENGTH, || {
        HeaderValue::from(body_len)
    });
}

impl<Connector> Client<Connector>
where
    Connector: Clone + Connect,
    for<'a> &'a HyperClient<Connector, BytesBody>:
        Service<http::Request<BytesBody>, Response = hyper::Response<Incoming>, Error: Into<Error>>,
{
    #[inline]
    pub fn new_from_connector(connector: Connector) -> Self {
        Self {
            client: HyperClient::builder(rt::TokioExecutor::new()).build(connector),
        }
    }

    pub async fn request(&self, mut request: http::Request<BytesBody>) -> Result<Bytes, Error> {
        insert_headers_for_request(&mut request);

        let mut response = self.request_with_retry(&request).await?;

        // handle at most 1 redirect
        if response.status().is_redirection() {
            *request.uri_mut() = get_redirect_uri(request.uri(), response)?;
            response = self.request_with_retry(&request).await?;
        }

        if !response.status().is_success() {
            let error =
                crate::ResponseError::from_response(request.uri().clone(), response).await?;
            return Err(Error::Response(error));
        }

        crate::util::collect_body(response.into_body())
            .await
            .map_err(Error::from)
    }

    pub async fn request_json<T>(&self, request: http::Request<BytesBody>) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let bytes = self.request(request).await?;

        path_aware_serde::json::deserialize_slice(&bytes).map_err(Error::Json)
    }

    async fn request_with_retry(
        &self,
        request: &http::Request<BytesBody>,
    ) -> Result<hyper::Response<Incoming>, Error> {
        enum ErrorKind {
            Response(http::Response<Incoming>),
            Error(Error),
        }

        macro_rules! make_request {
            () => {{
                match self.request_inner(request.clone()).await {
                    Ok(resp) if resp.status().is_server_error() => ErrorKind::Response(resp),
                    Ok(resp) => return Ok(resp),
                    Err(error) => ErrorKind::Error(error),
                }
            }};
        }

        // make an initial request before building any retry stuff up,
        // since it's likely to work first try (assuming a valid request)
        let mut error = make_request!();

        if cfg!(debug_assertions) {
            return match error {
                ErrorKind::Error(error) => Err(error),
                ErrorKind::Response(resp) => {
                    let response_error =
                        crate::ResponseError::from_response(request.uri().clone(), resp).await?;
                    Err(Error::Response(response_error))
                }
            };
        }

        let mut backoff = net_utils::backoff::Backoff::default();

        while let Some(backoff_delay) = backoff.backoff_once() {
            backoff_delay.await;
            error = make_request!();
        }

        match error {
            ErrorKind::Error(error) => Err(error),
            ErrorKind::Response(resp) => {
                let response_error =
                    crate::ResponseError::from_response(request.uri().clone(), resp).await?;
                Err(Error::Response(response_error))
            }
        }
    }

    async fn request_inner(
        &self,
        request: http::Request<BytesBody>,
    ) -> Result<hyper::Response<Incoming>, Error> {
        tower::Service::call(&mut &self.client, request)
            .await
            .map_err(Into::into)
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub(crate) struct BytesBody {
        bytes: Option<Bytes>,
    }
}

impl BytesBody {
    pub(crate) fn empty() -> Self {
        Self { bytes: None }
    }

    pub(crate) fn new(bytes: Bytes) -> Self {
        Self { bytes: Some(bytes) }
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.as_ref().map(Bytes::len).unwrap_or(0)
    }
}

impl Body for BytesBody {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.project().bytes.take() {
            Some(buf) if !buf.is_empty() => Poll::Ready(Some(Ok(Frame::data(buf)))),
            _ => Poll::Ready(None),
        }
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.bytes.is_none()
    }
}

fn get_redirect_uri(uri: &http::Uri, resp: http::Response<Incoming>) -> Result<http::Uri, Error> {
    debug_assert!(resp.status().is_redirection());

    let uri_header = match resp.headers().get(&http::header::LOCATION) {
        Some(header) => header,
        None => {
            return Err(Error::Response(crate::ResponseError::from_parts(
                uri.clone(),
                resp.into_parts().0,
                Bytes::from_static(b"recieved redirect response with no 'location' header"),
            )));
        }
    };

    match http::Uri::try_from(uri_header.as_bytes()) {
        Ok(uri) => Ok(uri),
        Err(error) => Err(Error::Response(crate::ResponseError::from_parts(
            uri.clone(),
            resp.into_parts().0,
            Bytes::from(error.to_string()),
        ))),
    }
}
