use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::response::IntoResponse;
use http_body::Body;
use tower::{Layer, Service};
use tracing::Span;

use crate::subscriber::SubscriberHandle;

#[derive(Debug, Clone)]
pub struct TraceLayer {
    handle: SubscriberHandle,
}

impl TraceLayer {
    pub const fn new(handle: SubscriberHandle) -> Self {
        Self { handle }
    }
}

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, svc: S) -> Self::Service {
        TraceService {
            svc,
            handle: self.handle.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceService<S> {
    svc: S,
    handle: SubscriberHandle,
}

impl<S, Rb> tower::Service<http::Request<Rb>> for TraceService<S>
where
    Rb: Body,
    S: Service<http::Request<Rb>>,
    S::Response: IntoResponse,
    S::Error: Into<Infallible>,
{
    type Error = Infallible;

    type Response = axum::response::Response;

    type Future = TraceFuture<S::Future, S::Response, S::Error>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: http::Request<Rb>) -> Self::Future {
        let span = crate::span::make_span(&req, &self.handle);
        let start = Instant::now();
        let fut = self.svc.call(req);

        TraceFuture {
            start,
            span,
            fut,
            handle: self.handle.clone(),
            _marker: PhantomData,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct TraceFuture<F, Resp, Err> {
        #[pin]
        fut: F,
        span: Span,
        start: Instant,
        handle: SubscriberHandle,
        _marker: PhantomData<fn(Resp, Err)>,
    }
}

impl<F, Resp, Err> Future for TraceFuture<F, Resp, Err>
where
    F: Future<Output = Result<Resp, Err>>,
    Err: Into<Infallible>,
    Resp: IntoResponse,
{
    type Output = Result<axum::response::Response, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // need to enter the span every time we poll
        let _guard = this.span.enter();

        let resp = match std::task::ready!(this.fut.poll(cx)).map_err(Into::<Infallible>::into) {
            Ok(res) => res,
            Err(infallible) => match infallible {},
        };

        let elapsed = this.start.elapsed();

        let response = resp.into_response();

        if let Some(span_id) = this.span.id() {
            this.handle
                .update_trace(&span_id, elapsed.into(), &response);
        }

        Poll::Ready(Ok(response))
    }
}
