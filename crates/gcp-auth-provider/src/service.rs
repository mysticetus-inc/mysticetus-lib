use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::StatusCode;
use net_utils::http_svc::{HttpRequest, HttpResponse};

use super::Auth;
use crate::cache::{GetHeaderResult, RefreshHeaderFuture, TokenCache};

#[derive(Debug, Clone, PartialEq)]
pub struct AuthSvc<Svc> {
    pub(super) auth: Auth,
    pub(super) svc: Svc,
}

impl<Svc> AuthSvc<Svc> {
    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    #[inline]
    pub fn map<Svc2>(self, map_fn: impl FnOnce(Svc) -> Svc2) -> AuthSvc<Svc2> {
        AuthSvc {
            auth: self.auth,
            svc: map_fn(self.svc),
        }
    }

    pub fn wrap_layer<L: tower::Layer<Svc> + ?Sized>(self, layer: &L) -> AuthSvc<L::Service> {
        let Self { auth, svc } = self;
        AuthSvc {
            auth,
            svc: layer.layer(svc),
        }
    }
}

impl<Svc, Req> tower::Service<Req> for AuthSvc<Svc>
where
    Svc: tower::Service<Req> + Clone,
    Req: HttpRequest,
    Svc::Response: HttpResponse,
{
    type Error = ServiceError<Svc::Error>;
    type Response = Svc::Response;
    type Future = ServiceFuture<'static, Req, Svc>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<core::result::Result<(), Self::Error>> {
        self.svc.poll_ready(cx).map_err(ServiceError::Service)
    }

    fn call(&mut self, mut req: Req) -> Self::Future {
        ServiceFuture {
            state: match self.auth.get_header() {
                GetHeaderResult::Cached(cached) => {
                    req.headers_mut()
                        .insert(http::header::AUTHORIZATION, cached.header);

                    ServiceFutureState::Calling {
                        auth: std::borrow::Cow::Owned(Arc::clone(&self.auth.inner)),
                        fut: self.svc.call(req),
                    }
                }
                GetHeaderResult::Refreshing(refresh) => ServiceFutureState::Authenticating {
                    refresh: refresh.into_static(),
                    parts: Some((req, self.svc.clone())),
                },
            },
        }
    }
}

impl<'a, Svc, Req> tower::Service<Req> for &'a AuthSvc<Svc>
where
    &'a Svc: tower::Service<Req>,
    Req: HttpRequest,
    <&'a Svc as tower::Service<Req>>::Response: HttpResponse,
{
    type Error = ServiceError<<&'a Svc as tower::Service<Req>>::Error>;
    type Response = <&'a Svc as tower::Service<Req>>::Response;
    type Future = ServiceFuture<'a, Req, &'a Svc>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<core::result::Result<(), Self::Error>> {
        (&mut &self.svc)
            .poll_ready(cx)
            .map_err(ServiceError::Service)
    }

    fn call(&mut self, mut req: Req) -> Self::Future {
        ServiceFuture {
            state: match self.auth.get_header() {
                GetHeaderResult::Cached(cached) => {
                    req.headers_mut()
                        .insert(http::header::AUTHORIZATION, cached.header);

                    ServiceFutureState::Calling {
                        auth: std::borrow::Cow::Owned(Arc::clone(&self.auth.inner)),
                        fut: (&mut &self.svc).call(req),
                    }
                }
                GetHeaderResult::Refreshing(refresh) => ServiceFutureState::Authenticating {
                    refresh: refresh.into_static(),
                    parts: Some((req, &self.svc)),
                },
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError<SvcError> {
    #[error(transparent)]
    Service(SvcError),
    #[error(transparent)]
    Auth(#[from] crate::Error),
}

pin_project_lite::pin_project! {
    pub struct ServiceFuture<'a, Req, Svc: tower::Service<Req>> {
        #[pin]
        state: ServiceFutureState<'a, Req, Svc>,
    }
}

pin_project_lite::pin_project! {
    #[project = ServiceFutureStateProjection]
    enum ServiceFutureState<'a, Req, Svc: tower::Service<Req>> {
        Authenticating {
            #[pin]
            refresh: RefreshHeaderFuture<'a>,
            parts: Option<(Req, Svc)>,
        },
        Calling {
            auth: std::borrow::Cow<'a, Arc<dyn TokenCache>>,
            #[pin]
            fut: Svc::Future,
        }
    }
}

impl<Req, Svc> Future for ServiceFuture<'_, Req, Svc>
where
    Svc: tower::Service<Req> + Clone,
    Req: HttpRequest,
    Svc::Response: HttpResponse,
{
    type Output = std::result::Result<Svc::Response, ServiceError<Svc::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        use ServiceFutureStateProjection::{Authenticating, Calling};

        loop {
            match this.state.as_mut().project() {
                Calling { fut, auth } => {
                    let resp = std::task::ready!(fut.poll(cx)).map_err(ServiceError::Service)?;

                    if resp.status() == StatusCode::UNAUTHORIZED {
                        auth.revoke(crate::cache::StartNewRequestOnRevoke::Yes);
                    }

                    return Poll::Ready(Ok(resp));
                }
                Authenticating { mut refresh, parts } => {
                    let creds =
                        std::task::ready!(refresh.as_mut().poll(cx)).map_err(ServiceError::Auth)?;
                    let (mut req, mut svc) = parts.take().expect("invalid state");

                    req.headers_mut()
                        .insert(http::header::AUTHORIZATION, creds.header);

                    let auth = std::borrow::Cow::Owned(refresh.get_mut().auth().clone());

                    this.state.as_mut().set(ServiceFutureState::Calling {
                        auth,
                        fut: svc.call(req),
                    });
                }
            }
        }
    }
}
