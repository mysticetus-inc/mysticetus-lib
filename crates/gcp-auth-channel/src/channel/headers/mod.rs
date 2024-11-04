use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};

use tonic::service::interceptor;
use tonic::{Request, Status};
use tower::{Layer, Service};

mod builder;
pub mod default;
mod kvp;

pub use builder::WithHeaderBuilder;
pub use kvp::{Grpc, Http, InsertHeaders, KeyValuePair};

use super::AuthChannel;

/// Holds a generic [`KeyValuePair`] pair.
///
/// [`WithHeader`] is the associated [`Service`] that this generates via [`Layer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithHeaderLayer<Kvp: KeyValuePair> {
    key: Kvp::Key,
    value: Kvp::Value,
}

impl interceptor::Interceptor for WithHeaderLayer<Grpc> {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());
        Ok(request)
    }
}

/// A service that wraps another [`Service`]. Inserts a header into the request,
/// then passes it to the inner [`Service`].
///
/// [`WithHeaderLayer`] is the associated [`Layer`].
#[derive(Debug, Clone)]
pub struct WithHeader<S, Kvp: KeyValuePair> {
    pub(super) service: S,
    pub(super) key: Kvp::Key,
    pub(super) value: Kvp::Value,
}

// helper macro for defining common getters on the With(Default)Header(s) services and associated
// layers.
macro_rules! impl_with_header_shared_fns {
    ($kvp:ty; $($name:tt).+ => $($value:tt).+) => {
        pub fn name(&self) -> &<$kvp as KeyValuePair>::Key {
            &self.$($name).+
        }

        pub fn value(&self) -> &<$kvp as KeyValuePair>::Value {
            &self.$($value).+
        }
    };
    ($kvp:ty => with mut; $($name:tt).+ => $($value:tt).+) => {
        impl_with_header_shared_fns!($kvp; $($name).+ => $($value).+);

        pub fn name_mut(&mut self) -> &mut <$kvp as KeyValuePair>::Key {
            &mut self.$($name).+
        }

        pub fn value_mut(&mut self) -> &mut <$kvp as KeyValuePair>::Value {
            &mut self.$($value).+
        }
    };
}

use impl_with_header_shared_fns;

impl<Kvp> WithHeaderLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    impl_with_header_shared_fns!(Kvp => with mut; key => value);

    pub fn into_default(self) -> default::WithDefaultHeaderLayer<Kvp> {
        default::WithDefaultHeaderLayer {
            pair: Arc::new((self.key, self.value)),
        }
    }

    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<Kvp::Key>,
        V: Into<Kvp::Value>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn new_from_str<K, V, E>(key: K, value: V) -> Result<Self, E>
    where
        K: AsRef<str>,
        V: AsRef<str>,
        Kvp::Key: FromStr<Err = E>,
        Kvp::Value: FromStr<Err = E>,
    {
        let key = key.as_ref().parse()?;
        let value = value.as_ref().parse()?;

        Ok(Self { key, value })
    }

    pub fn try_new<K, V, E>(key: K, value: V) -> Result<Self, E>
    where
        K: TryInto<Kvp::Key, Error = E>,
        V: TryInto<Kvp::Value, Error = E>,
    {
        let value = value.try_into()?;
        let key = key.try_into()?;

        Ok(Self { key, value })
    }

    /// Identical to [`Layer::layer`], but without requiring the trait be in scope.
    #[inline]
    pub fn layer<S>(&self, service: AuthChannel<S>) -> AuthChannel<WithHeader<S, Kvp>> {
        Layer::layer(self, service)
    }
}

impl<S, Kvp> WithHeader<S, Kvp>
where
    Kvp: KeyValuePair,
{
    impl_with_header_shared_fns!(Kvp => with mut; key => value);

    pub fn builder(service: AuthChannel<S>) -> WithHeaderBuilder<S, Kvp, (), ()> {
        WithHeaderBuilder::from_service(service)
    }

    pub fn into_default(self) -> default::WithDefaultHeader<S, Kvp> {
        default::WithDefaultHeader {
            pair: Arc::new((self.key, self.value)),
            service: self.service,
        }
    }
}
/*
impl<S> WithHeader<S, Grpc> {
    pub fn into_intercepted(
        self,
    ) -> interceptor::InterceptedService<AuthChannel<S>, WithHeaderLayer<Grpc>> {
        interceptor::InterceptedService::new(
            self.service,
            WithHeaderLayer {
                key: self.key,
                value: self.value,
            },
        )
    }
}
*/

impl<S, Kvp> Layer<AuthChannel<S>> for WithHeaderLayer<Kvp>
where
    Kvp: KeyValuePair,
{
    type Service = AuthChannel<WithHeader<S, Kvp>>;

    fn layer(&self, service: AuthChannel<S>) -> Self::Service {
        AuthChannel {
            auth: service.auth,
            svc: WithHeader {
                key: self.key.clone(),
                value: self.value.clone(),
                service: service.svc,
            },
        }
    }
}

impl<S, Kvp, Req> Service<Req> for WithHeader<S, Kvp>
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
        req.insert_header(self.key.clone(), self.value.clone());
        self.service.call(req)
    }
}
