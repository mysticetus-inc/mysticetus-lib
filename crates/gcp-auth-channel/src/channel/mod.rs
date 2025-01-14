//! [`AuthChannel`], a [`Service`] that inserts authentication details.
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{Request, header};
use tonic::transport::Channel;
use tower::{Layer, Service};

use crate::auth::GetHeaderResult;
use crate::scope::Scope;
use crate::{Auth, Error};

mod builder;
use builder::AuthChannelBuilder;
mod future;
pub mod headers;
pub mod retry;

use headers::KeyValuePair;

/// Constructs a user agent string in the form:
/// `mysticetus-{PKG_NAME}-{PKG_VERSION}`
#[macro_export]
macro_rules! user_agent {
    () => {
        concat!(
            "mystiectus-",
            env!("CARGO_PKG_NAME"),
            "-",
            env!("CARGO_PKG_VERSION")
        )
    };
}

/// Alias for a pinned + boxed type erased [`Service`].
pub type BoxedService<Req, Resp, Err, Fut> =
    Pin<Box<dyn Service<Req, Response = Resp, Error = Err, Future = Fut>>>;

/// A bare channel, authenticated to a [Scope] determined by the internal [Auth]
#[derive(Debug, Clone)]
pub struct AuthChannel<Svc = Channel> {
    svc: Svc,
    auth: Auth,
}

impl AuthChannel {
    pub fn builder() -> AuthChannelBuilder {
        AuthChannelBuilder::new()
    }

    pub async fn from_static(
        project_id: &'static str,
        url: &'static str,
        scope: Scope,
    ) -> Result<Self, Error> {
        async fn build_channel(url: &'static str) -> Result<Channel, Error> {
            Channel::from_static(url)
                .connect()
                .await
                .map_err(|e| Error::Transport(e.into()))
        }

        let (channel, auth) = tokio::try_join!(build_channel(url), Auth::new(project_id, scope))?;

        let auth_channel = Self::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        Ok(auth_channel)
    }
}

impl<Svc: Clone> AuthChannel<Svc> {
    /// This function is to get around the normal clone behavior, in order to avoid the panic
    /// caused by tonic using a tower buffer internally.
    ///
    /// See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149 for details
    fn clone_parts(&mut self) -> Self {
        let clone = self.svc.clone();

        let replaced = std::mem::replace(&mut self.svc, clone);

        Self {
            auth: self.auth.clone(),
            svc: replaced,
        }
    }
}

impl<Svc> AuthChannel<Svc> {
    /// Returns a reference to the internal [`Arc`] holding the [`Auth`] instance. This way,
    /// we can clone it via [`Arc::clone`] if needed, and just treat it like a reference
    /// if we dont.
    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    /// Converts the inner [Auth] to use a new scope.
    pub fn with_scope(self, scope: Scope) -> Self {
        let auth = self.auth.with_new_scope(scope);
        Self {
            svc: self.svc,
            auth,
        }
    }

    #[inline]
    pub fn into_service(self) -> Svc {
        self.svc
    }

    #[inline]
    pub fn service(&self) -> &Svc {
        &self.svc
    }

    #[inline]
    pub fn service_mut(&mut self) -> &mut Svc {
        &mut self.svc
    }

    /// Returns a builder that'll wrap [`Svc`] with a new layer, inserting a header into each
    /// request.
    pub fn attach_headers<Kvp>(
        self,
        headers: impl Into<Arc<[(Kvp::Key, Kvp::Value)]>>,
    ) -> headers::WithHeaders<Self, Kvp>
    where
        Kvp: KeyValuePair,
    {
        headers::WithHeaders::new(self, headers)
    }

    /// Wraps the inner [`Svc`], replacing it with the returned value of 'f'.
    ///
    /// Similar to [`apply_layer`], but is a bit more flexible.
    ///
    /// [`apply_layer`]: AuthChannel::apply_layer
    #[inline]
    pub fn wrap_service<F, S2>(self, f: F) -> AuthChannel<S2>
    where
        F: FnOnce(Svc) -> S2,
    {
        AuthChannel {
            svc: f(self.svc),
            auth: self.auth,
        }
    }

    /// Applys a [`Layer`] to the inner `Svc`, and returns 'self' with the wrapped
    /// [`Layer::Service`].
    ///
    /// Similar to [`wrap_service`], but a bit more convienient for [`Layer`] types.
    ///
    /// [`wrap_service`]: AuthChannel::wrap_service
    #[inline]
    pub fn apply_layer<L>(self, layer: L) -> AuthChannel<L::Service>
    where
        L: Layer<Svc>,
    {
        AuthChannel {
            svc: layer.layer(self.svc),
            auth: self.auth,
        }
    }

    #[cfg(feature = "retry")]
    pub fn with_retry<P, Body>(self, policy: P) -> AuthChannel<tower::retry::Retry<P, Svc>>
    where
        // technically these bounds aren't required here, but this will give better
        // errors if request/response/error types don't line up, rather than running into that
        // when trying to invoke this as a [`Service`].
        Svc: Service<Request<Body>>,
        P: tower::retry::Policy<Request<Body>, Svc::Response, Svc::Error>,
    {
        AuthChannel {
            auth: self.auth,
            svc: tower::retry::Retry::new(policy, self.svc),
        }
    }

    pub fn with_timeout(
        self,
        timeout: timestamp::Duration,
    ) -> AuthChannel<tower::timeout::Timeout<Svc>> {
        let l = tower::timeout::TimeoutLayer::new(timeout.into());
        self.apply_layer(l)
    }

    // dont really have too much of a choice, we need to specify each associated type for the
    // resulting service to be object safe.
    #[allow(clippy::type_complexity)]
    /// Turns 'Svc' into a [`Pin<Box<dyn Service<_>>>`], providing type erasure + [`Unpin`].
    pub fn boxed<Req>(
        self,
    ) -> AuthChannel<BoxedService<Req, Svc::Response, Svc::Error, Svc::Future>>
    where
        Svc: Service<Req> + 'static,
    {
        AuthChannel {
            auth: self.auth,
            svc: Box::pin(self.svc),
        }
    }
}

#[inline]
fn make_future_state<Svc, Body>(
    svc: &mut Svc,
    auth: &Auth,
    mut req: Request<Body>,
) -> future::State<Svc, Body>
where
    Svc: Service<Request<Body>>,
    Svc::Future: Unpin,
    Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    // Try and get an existing non-expired token to skip a step.
    match auth.get_header() {
        GetHeaderResult::Cached(header) => {
            req.headers_mut().insert(header::AUTHORIZATION, header);

            future::State::MakingRequest {
                future: Service::call(svc, req),
            }
        }
        GetHeaderResult::Refreshing(future) => future::State::GettingToken {
            future,
            req: Some(req),
        },
    }
}

impl<Svc, Body> Service<Request<Body>> for AuthChannel<Svc>
where
    Svc: Service<Request<Body>> + Clone,
    Svc::Future: Unpin,
    Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Svc::Response;
    type Error = Error;
    type Future = future::AuthFuture<Svc, Body>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.svc, cx).map_err(|e| Error::Transport(e.into()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // see docs on `clone_parts` as to why this is needed
        let mut new_self = self.clone_parts();

        future::AuthFuture {
            state: make_future_state(&mut new_self.svc, &new_self.auth, req),
            channel: new_self,
        }
    }
}

impl<'a, Svc, Body> Service<Request<Body>> for &'a AuthChannel<Svc>
where
    &'a Svc: Service<Request<Body>>,
    <&'a Svc as Service<Request<Body>>>::Future: Unpin,
    <&'a Svc as Service<Request<Body>>>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = <&'a Svc as Service<Request<Body>>>::Response;
    type Error = Error;
    type Future = future::AuthFuture<&'a Svc, Body>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut &self.svc, cx).map_err(|e| Error::Transport(e.into()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Try and get an existing non-expired token to skip a step.
        future::AuthFuture {
            state: make_future_state(&mut &self.svc, &self.auth, req),
            channel: AuthChannel {
                svc: &self.svc,
                auth: self.auth.clone(),
            },
        }
    }
}
