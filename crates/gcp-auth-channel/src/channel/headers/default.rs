//! Layer/Service definitions that add default headers to each request.

use std::sync::Arc;
use std::task::{Context, Poll};

use tower::{Layer, Service};

use super::{AuthChannel, InsertHeaders, KeyValuePair, impl_with_header_shared_fns};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithDefaultHeaderLayer<Kvp: KeyValuePair> {
    pub(super) pair: Arc<(Kvp::Key, Kvp::Value)>,
}

impl<Kvp> WithDefaultHeaderLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    impl_with_header_shared_fns!(Kvp; pair.0 => pair.1);
}

#[derive(Debug, Clone)]
pub struct WithDefaultHeader<S, Kvp: KeyValuePair> {
    pub(super) pair: Arc<(Kvp::Key, Kvp::Value)>,
    pub(super) service: S,
}

impl<S, Kvp> WithDefaultHeader<S, Kvp>
where
    Kvp: KeyValuePair,
{
    impl_with_header_shared_fns!(Kvp; pair.0 => pair.1);
}

impl<S, Kvp> Layer<AuthChannel<S>> for WithDefaultHeaderLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    type Service = AuthChannel<WithDefaultHeader<S, Kvp>>;

    fn layer(&self, service: AuthChannel<S>) -> Self::Service {
        AuthChannel {
            auth: service.auth,
            svc: WithDefaultHeader {
                pair: Arc::clone(&self.pair),
                service: service.svc,
            },
        }
    }
}

impl<S, Kvp, Req> Service<Req> for WithDefaultHeader<S, Kvp>
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
        req.insert_header(self.pair.0.clone(), self.pair.1.clone());
        self.service.call(req)
    }
}

impl<Kvp> From<WithDefaultHeaderLayer<Kvp>> for super::WithHeaderLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    /// Tries to unwrap the Arc to avoid an allocation, but falls back on cloning if needed
    fn from(def: WithDefaultHeaderLayer<Kvp>) -> Self {
        match Arc::try_unwrap(def.pair) {
            Ok((key, value)) => super::WithHeaderLayer { key, value },
            Err(arc) => super::WithHeaderLayer {
                key: arc.0.clone(),
                value: arc.1.clone(),
            },
        }
    }
}
