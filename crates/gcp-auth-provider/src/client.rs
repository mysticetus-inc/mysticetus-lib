use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http::HeaderValue;
use http_body::{Body, Frame};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::client::legacy::connect::dns::GaiResolver;
use hyper_util::client::legacy::connect::{self, HttpConnector};
use hyper_util::rt;

pub mod future;

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

/// Trait alias for connectors that allow [`&'_ Client<C: Connector, B>`] to
/// implement: [`tower::Service<http::Request<B>>`]
pub(crate) trait Connector: connect::Connect + Clone + Send + Sync + 'static {}

impl<C> Connector for C where C: connect::Connect + Clone + Send + Sync + 'static {}

impl<Conn> Client<Conn>
where
    Conn: Connector,
{
    #[inline]
    pub fn new_from_connector(connector: Conn) -> Self {
        Self {
            client: HyperClient::builder(rt::TokioExecutor::new()).build(connector),
        }
    }

    pub fn request(
        &self,
        mut request: http::Request<BytesBody>,
    ) -> future::RequestCollect<'_, Conn> {
        insert_headers_for_request(&mut request);

        future::RequestCollect::Requesting {
            request: future::Request::new(&self.client, request),
        }
    }

    pub fn request_json<T>(
        &self,
        request: http::Request<BytesBody>,
    ) -> future::RequestJson<'_, Conn, T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.request(request).json()
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

fn make_https_connection<R>(resolver: R) -> std::io::Result<HttpsConnector<HttpConnector<R>>> {
    static INIT_RUSTLS: std::sync::Once = std::sync::Once::new();

    INIT_RUSTLS.call_once(|| {
        _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });

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
