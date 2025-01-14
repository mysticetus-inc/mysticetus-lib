use std::sync::Arc;
use std::task::{Context, Poll};

use tonic::service::interceptor;
use tonic::{Request, Status};
use tower::{Layer, Service};

mod kvp;

pub use kvp::{Grpc, Http, InsertHeaders, KeyValuePair};

use super::AuthChannel;

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
