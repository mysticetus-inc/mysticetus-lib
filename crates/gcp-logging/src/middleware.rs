use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::response::{IntoResponse, Response};
use futures::TryFuture;
use http_body::Body;
use tower::{Layer, Service};
use tracing::Span;

use crate::http_request::{RESPONSE_LATENCY_KEY, RESPONSE_SIZE_KEY, RESPONSE_STATUS_KEY};
use crate::subscriber::Handle;

tokio::task_local! {
    static REQUEST_SPAN: Option<tracing::Id>;
}

pub fn current_request_span_id() -> Option<tracing::Id> {
    REQUEST_SPAN
        .try_with(|maybe_id| maybe_id.clone())
        .ok()
        .flatten()
}

impl<S> Layer<S> for Handle {
    type Service = TraceService<S>;

    fn layer(&self, svc: S) -> Self::Service {
        TraceService {
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
        use tracing::field::Empty;

        use crate::registry::{NewRequest, REQUEST_KEY};
        use crate::utils::ErrorPassthrough;

        let span = tracing::info_span! {
            "request",
            { REQUEST_KEY } = ErrorPassthrough(NewRequest::new(&req)).as_dyn(),
            { RESPONSE_LATENCY_KEY } = Empty,
            { RESPONSE_SIZE_KEY } = Empty,
            { RESPONSE_STATUS_KEY } = Empty,
        };

        // need to enter the scope not just when polling, but here too.
        // 'Service::call' likely does work that may include logging,
        // which we (obviously) want to include

        let (fut, started) = in_scope(&span, || {
            let started = if span.is_disabled() {
                None
            } else {
                Some(Instant::now())
            };

            (self.svc.call(req), started)
        });

        TraceFuture { fut, span, started }
    }
}

#[inline]
fn in_scope<O>(span: &tracing::Span, f: impl FnOnce() -> O) -> O {
    REQUEST_SPAN.sync_scope(span.id(), || {
        let _guard = span.enter();
        f()
    })
}

pin_project_lite::pin_project! {
    #[project = TraceFutureProjection]
    pub struct TraceFuture<F: TryFuture> {
        #[pin]
        fut: F,
        span: Span,
        started: Option<Instant>,
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
        let TraceFutureProjection { fut, span, started } = self.project();

        in_scope(span, || {
            let Ok(resp) = std::task::ready!(fut.poll(cx));
            let resp = resp.into_response();

            record_response_fields(span, &resp, started.take());

            Poll::Ready(Ok(resp))
        })
    }
}

fn record_response_fields<B: Body>(
    span: &tracing::Span,
    resp: &http::Response<B>,
    started: Option<Instant>,
) {
    use tracing::field::{AsField, Field, Value};

    struct GoogleDurationFormat(Duration);

    impl std::fmt::Display for GoogleDurationFormat {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}s", self.0.as_secs_f64())
        }
    }

    let Some(metadata) = span.metadata() else {
        return;
    };

    let status = resp.status().as_u16() as u64;

    let status_field: Field;
    let status_pair = if let Some(field) = RESPONSE_STATUS_KEY.as_field(metadata) {
        status_field = field;
        Some((&status_field, Some(&status as &dyn Value)))
    } else {
        None
    };

    let response_size = crate::http_request::get_response_size(resp);

    let size: u64;
    let size_field: Field;
    let resp_size_pair = if let Some(s) = response_size.get()
        && let Some(size_f) = RESPONSE_SIZE_KEY.as_field(metadata)
    {
        size = s;
        size_field = size_f;
        Some((&size_field, Some(&size as &dyn Value)))
    } else {
        None
    };

    let latency_field: Field;
    let latency: tracing::field::DisplayValue<GoogleDurationFormat>;
    let latency_pair = if let Some(started) = started
        && let Some(field) = RESPONSE_LATENCY_KEY.as_field(metadata)
    {
        latency_field = field;
        latency = tracing::field::display(GoogleDurationFormat(started.elapsed()));
        Some((&latency_field, Some(&latency as &dyn Value)))
    } else {
        None
    };

    let two: [(&Field, Option<&dyn Value>); 2];
    let three: [(&Field, Option<&dyn Value>); 3];

    span.record_all(&match (status_pair, resp_size_pair, latency_pair) {
        // since each pair is already cast to the resulting dyn Value, we
        // can simplify the match arms by ignoring the specifics of each pair
        (Some(a), Some(b), Some(c)) => {
            three = [a, b, c];
            metadata.fields().value_set(&three)
        }
        (Some(a), Some(b), None) | (Some(a), None, Some(b)) | (None, Some(a), Some(b)) => {
            two = [a, b];
            metadata.fields().value_set(&two)
        }
        (Some(ref a), None, None) | (None, Some(ref a), None) | (None, None, Some(ref a)) => {
            metadata.fields().value_set(std::array::from_ref(a))
        }
        // if there's nothing to record, don't bother trying to record an empty
        // array of values
        (None, None, None) => return,
    });
}
