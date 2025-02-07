use http_body::Body;

use crate::subscriber::{RequestTrace, SubscriberHandle};

pub const SPAN_FIELD_NAME: &str = "logging.googleapis.com/spanId";
pub const TRACE_FIELD_NAME: &str = "logging.googleapis.com/trace";

pub(super) fn make_span<B: Body>(
    request: &http::Request<B>,
    handle: &SubscriberHandle,
) -> tracing::Span {
    let span = tracing::info_span!("request");

    if let Some(span_id) = span.id() {
        handle.insert_new_trace(RequestTrace::new(span_id, request));
    }

    span
}
