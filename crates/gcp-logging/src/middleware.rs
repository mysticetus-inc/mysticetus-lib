use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::response::{IntoResponse, Response};
use futures::{TryFuture, future};
use http_body::Body;
use tower::{Layer, Service};
use tracing::Span;

use crate::records::ActiveRequest;
use crate::subscriber::{Handle, WeakHandle};

impl<S> Layer<S> for Handle {
    type Service = TraceService<S>;

    fn layer(&self, svc: S) -> Self::Service {
        TraceService {
            svc,
            handle: self.clone(),
        }
    }
}

impl<S> Layer<S> for WeakHandle {
    type Service = WeakTraceService<S>;

    fn layer(&self, svc: S) -> Self::Service {
        WeakTraceService {
            svc,
            handle: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceService<S> {
    svc: S,
    handle: Handle,
}

#[derive(Debug, Clone)]
pub struct WeakTraceService<S> {
    svc: S,
    handle: WeakHandle,
}

impl<S> WeakTraceService<S> {
    pub fn upgrade_ref(&mut self) -> Option<TraceService<&mut S>> {
        self.handle.upgrade().map(|handle| TraceService {
            svc: &mut self.svc,
            handle,
        })
    }

    pub fn upgrade(self) -> Result<TraceService<S>, Self> {
        match self.handle.upgrade() {
            Some(handle) => Ok(TraceService {
                svc: self.svc,
                handle,
            }),
            None => Err(self),
        }
    }
}

// The bounds need to be essentially the same as the bounds for axum::Router::layer:
// https://docs.rs/axum/latest/axum/routing/struct.Router.html#method.layer
impl<S, RequestBody> tower::Service<http::Request<RequestBody>> for TraceService<S>
where
    RequestBody: Body,
    S: Service<http::Request<RequestBody>, Error = Infallible>,
    S::Response: IntoResponse,
{
    type Error = Infallible;

    type Response = Response;

    type Future = TraceFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: http::Request<RequestBody>) -> Self::Future {
        let span = tracing::info_span!("request");
        let active = self.handle.start_request_for_span(&span, &req);

        // need to enter the scope not just when polling, but here too.
        // 'Service::call' likely does work that may include logging,
        // which we (obviously) want to include
        let fut = span.in_scope(|| self.svc.call(req));

        TraceFuture { span, fut, active }
    }
}

pin_project_lite::pin_project! {
    pub struct TraceFuture<F: TryFuture> {
        #[pin]
        fut: F,
        span: Span,
        active: Option<ActiveRequest>,
    }
}

impl<F: TryFuture> Future for TraceFuture<F>
where
    F: TryFuture + Future<Output = Result<F::Ok, Infallible>>,
    F::Ok: IntoResponse,
{
    type Output = Result<Response, Infallible>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // need to enter the span every time we poll
        let _guard = this.span.enter();

        let Ok(resp) = std::task::ready!(this.fut.poll(cx));
        let resp = resp.into_response();

        if let Some(active) = this.active.take() {
            active.update_from_response(&resp);
        }

        Poll::Ready(Ok(resp))
    }
}

pub type WeakTraceFuture<F> = future::Either<
    TraceFuture<F>,
    future::Map<F, fn(<F as Future>::Output) -> Result<Response, Infallible>>,
>;

// The bounds need to be essentially the same as the bounds for axum::Router::layer:
// https://docs.rs/axum/latest/axum/routing/struct.Router.html#method.layer
impl<S, RequestBody> tower::Service<http::Request<RequestBody>> for WeakTraceService<S>
where
    RequestBody: Body,
    S: Service<http::Request<RequestBody>, Error = Infallible>,
    S::Response: IntoResponse,
{
    type Error = Infallible;

    type Response = Response;

    type Future = WeakTraceFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: http::Request<RequestBody>) -> Self::Future {
        use futures::future::FutureExt;

        fn map_response<R: IntoResponse>(
            result: Result<R, Infallible>,
        ) -> Result<Response, Infallible> {
            result.map(R::into_response)
        }

        match self.upgrade_ref() {
            Some(mut svc) => future::Either::Left(svc.call(req)),
            None => future::Either::Right(self.svc.call(req).map(map_response)),
        }
    }
}
