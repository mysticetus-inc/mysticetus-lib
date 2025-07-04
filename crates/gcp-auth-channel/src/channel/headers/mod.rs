use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::BytesMut;
use tonic::service::interceptor;
use tonic::{Request, Status};
use tower::{Layer, Service};

mod kvp;

pub use kvp::{Grpc, Http, InsertHeaders, KeyValuePair};

use super::AuthChannel;

const GOOG_REQUEST_PARAMS: http::HeaderName =
    http::HeaderName::from_static("x-goog-request-params");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogRequestParams<Svc> {
    svc: Svc,
    value: http::HeaderValue,
}

impl<Svc> GoogRequestParams<Svc> {
    pub fn new_parse(
        svc: Svc,
        pairs: &[(&str, &str)],
    ) -> Result<Self, (Svc, http::header::InvalidHeaderValue)> {
        assert_ne!(
            0,
            pairs.len(),
            "need at least 1 pair to encode `x-goog-request-params` header value"
        );

        let raw_len = pairs
            .iter()
            .map(|(k, v)| k.len() + 1 + v.len())
            .sum::<usize>();

        let len = raw_len + pairs.len() - 1;

        let mut buf = BytesMut::with_capacity(len);

        for (idx, (k, v)) in pairs.iter().enumerate() {
            if idx > 0 {
                buf.extend_from_slice(b"&");
            }

            buf.extend_from_slice(k.as_bytes());
            buf.extend_from_slice(b"=");
            buf.extend_from_slice(v.as_bytes());
        }

        debug_assert_eq!(buf.len(), len);

        match http::HeaderValue::from_maybe_shared(buf.freeze()) {
            Ok(value) => Ok(Self { value, svc }),
            Err(error) => Err((svc, error)),
        }
    }

    pub fn new(svc: Svc, value: http::HeaderValue) -> Self {
        Self { svc, value }
    }
}

impl<Svc, Body> Service<http::Request<Body>> for GoogRequestParams<Svc>
where
    Svc: Service<http::Request<Body>>,
{
    type Error = Svc::Error;
    type Future = Svc::Future;
    type Response = Svc::Response;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut req: http::Request<Body>) -> Self::Future {
        req.headers_mut()
            .insert(GOOG_REQUEST_PARAMS, self.value.clone());
        self.svc.call(req)
    }
}

pub struct AddHeaderService<Svc> {
    pub(super) svc: Svc,
    pub(super) name: http::HeaderName,
    pub(super) value: http::HeaderValue,
}

impl<Svc, Body> Service<http::Request<Body>> for AddHeaderService<Svc>
where
    Svc: Service<http::Request<Body>>,
{
    type Error = Svc::Error;
    type Future = Svc::Future;
    type Response = Svc::Response;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut req: http::Request<Body>) -> Self::Future {
        req.headers_mut()
            .insert(self.name.clone(), self.value.clone());

        self.svc.call(req)
    }
}

/// Holds a generic [`KeyValuePair`] pair.
///
/// [`WithHeader`] is the associated [`Service`] that this generates via [`Layer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithHeadersLayer<Kvp: KeyValuePair> {
    pairs: Arc<[(Kvp::Key, Kvp::Value)]>,
}

impl interceptor::Interceptor for WithHeadersLayer<Grpc> {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let meta = request.metadata_mut();

        for (key, value) in self.pairs.iter() {
            meta.insert(key.clone(), value.clone());
        }

        Ok(request)
    }
}

/// A service that wraps another [`Service`]. Inserts a header into the request,
/// then passes it to the inner [`Service`].
///
/// [`WithHeaderLayer`] is the associated [`Layer`].
#[derive(Debug)]
pub struct WithHeaders<S, Kvp: KeyValuePair> {
    pub(super) service: S,
    pub(super) pairs: Arc<[(Kvp::Key, Kvp::Value)]>,
}

impl<S: Clone, Kvp: KeyValuePair> Clone for WithHeaders<S, Kvp> {
    fn clone(&self) -> Self {
        WithHeaders {
            service: self.service.clone(),
            pairs: Arc::clone(&self.pairs),
        }
    }
}

impl<S, Kvp: KeyValuePair> WithHeaders<S, Kvp> {
    pub fn new(service: S, pairs: impl Into<Arc<[(Kvp::Key, Kvp::Value)]>>) -> Self {
        Self {
            service,
            pairs: pairs.into(),
        }
    }
}

impl<Kvp: KeyValuePair> WithHeadersLayer<Kvp> {
    pub fn new_pair<K, V>(key: K, value: V) -> Self
    where
        K: Into<Kvp::Key>,
        V: Into<Kvp::Value>,
    {
        Self {
            pairs: Arc::from([(key.into(), value.into())]),
        }
    }

    pub fn new(pairs: impl Into<Arc<[(Kvp::Key, Kvp::Value)]>>) -> Self {
        Self {
            pairs: pairs.into(),
        }
    }

    /// Identical to [`Layer::layer`], but without requiring the trait be in scope.
    #[inline]
    pub fn layer<S>(&self, service: AuthChannel<S>) -> AuthChannel<WithHeaders<S, Kvp>> {
        Layer::layer(self, service)
    }
}

impl<S, Kvp> Layer<AuthChannel<S>> for WithHeadersLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    type Service = AuthChannel<WithHeaders<S, Kvp>>;

    fn layer(&self, service: AuthChannel<S>) -> Self::Service {
        AuthChannel {
            auth: service.auth,
            svc: WithHeaders {
                pairs: Arc::clone(&self.pairs),
                service: service.svc,
            },
        }
    }
}

impl<S, Kvp, Req> Service<Req> for WithHeaders<S, Kvp>
where
    S: Service<Req>,
    Kvp: KeyValuePair,
    Req: InsertHeaders<Kvp>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Req) -> Self::Future {
        for (key, value) in self.pairs.iter() {
            req.insert_header(key.clone(), value.clone());
        }
        self.service.call(req)
    }
}
