use std::sync::Arc;

use dashmap::DashMap;
use http::HeaderValue;
use http_body::Body;
use timestamp::Duration;
use tracing::span::Id;

use crate::http_request::{HttpRequest, TRACE_CTX_HEADER};

#[derive(Debug, Clone)]
pub struct RequestTrace {
    pub span_id: Id,
    pub trace_header: Option<HeaderValue>,
    pub request: HttpRequest,
}

impl RequestTrace {
    pub fn new<B: Body>(span_id: Id, request: &http::Request<B>) -> Self {
        Self {
            span_id,
            trace_header: request.headers().get(TRACE_CTX_HEADER).cloned(),
            request: HttpRequest::from_request(&request),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscriberHandle {
    trace_headers: Arc<DashMap<Id, RequestTrace, fxhash::FxBuildHasher>>,
}

impl SubscriberHandle {
    pub(crate) fn new(
        trace_headers: Arc<DashMap<Id, RequestTrace, fxhash::FxBuildHasher>>,
    ) -> Self {
        Self { trace_headers }
    }

    pub fn insert_new_trace(&self, trace: RequestTrace) {
        self.trace_headers.insert(trace.span_id.clone(), trace);
    }

    pub fn update_trace<B: Body>(
        &self,
        span_id: &Id,
        latency: Duration,
        response: &http::Response<B>,
    ) {
        if let Some(mut request_trace) = self.trace_headers.get_mut(span_id) {
            request_trace
                .request
                .update_from_response(latency, response);
        }
    }
}

/*
pub struct Subscriber<W = Stdout> {
    env_filter: EnvFilter,
    registry: Registry,
    trace_headers: Arc<DashMap<Id, RequestTrace, fxhash::FxBuildHasher>>,
    writer: W,
}

impl<W: 'static> tracing::subscriber::Subscriber for Subscriber<W> {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        todo!()
    }

    fn enter(&self, span: &tracing::span::Id) {
        todo!()
    }

    fn exit(&self, span: &tracing::span::Id) {
        todo!()
    }

    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        thread_local! {
            static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
        }

        let id = RNG.with(|rng| {
            let id = rng.borrow_mut().gen_range(1_u64..=u64::MAX);

            let id = NonZeroU64::new(id)
                .expect("gen_range(1_u64..) should always return a non-zero value");

            tracing::span::Id::from_non_zero_u64(id)
        });

        id
    }

    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing::Event<'_>) {
        todo!()
    }
}
*/
