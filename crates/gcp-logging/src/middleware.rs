use std::time::Duration;

use http::Request;
use http::header::{self, HeaderMap, HeaderName};
use http_body::Body;
use tower_http::classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier};
use tower_http::trace;
use tracing::field::Empty;

pub type Classifier = SharedClassifier<ServerErrorsAsFailures>;

pub type TraceLayer = trace::TraceLayer<
    Classifier,
    MakeSpan,
    OnRequest,
    OnResponse, // non-default
    trace::DefaultOnBodyChunk,
    trace::DefaultOnEos, // defaults
    OnFailure,           // non-default
>;

pub fn new_trace_layer() -> TraceLayer {
    trace::TraceLayer::new_for_http()
        .make_span_with(MakeSpan)
        .on_request(OnRequest)
        .on_response(OnResponse)
        .on_failure(OnFailure)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MakeSpan;

impl MakeSpan {
    pub const SPAN_NAME: &'static str = "__http_request";

    pub const SPAN_FIELDS: &'static [&'static str] = &[
        "__http_request.trace",
        "__http_request.method",
        "__http_request.request_url",
        "__http_request.referer",
        "__http_request.user_agent",
        "__http_request.protocol",
        "__http_request.response_size",
        "__http_request.request_size",
        "__http_request.status",
        "__http_request.content_type",
        "__http_request.latency.seconds",
        "__http_request.latency.nanos",
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnResponse;

impl<B> trace::MakeSpan<B> for MakeSpan
where
    B: Body,
{
    fn make_span(&mut self, req: &Request<B>) -> tracing::Span {
        let head = req.headers();

        tracing::info_span!(
            MakeSpan::SPAN_NAME,
            "__http_request.method" = req.method().as_str(),
            "__http_request.request_url" = %req.uri(),
            "__http_request.referer" = get_header_value_string(head, &header::REFERER),
            "__http_request.user_agent" = get_header_value_string(head, &header::USER_AGENT),
            "__http_request.protocol" = get_version_str(req.version()),
            "__http_request.request_size" = req.body().size_hint().exact(),
            "__http_request.status" = Empty,
            "__http_request.content_type" = get_header_value_string(head, &header::CONTENT_TYPE),
            "__http_request.latency.seconds" = Empty,
            "__http_request.latency.nanos" = Empty,
        )
    }
}

fn get_version_str(version: http::Version) -> Option<&'static str> {
    match version {
        http::Version::HTTP_09 => Some("HTTP/0.9"),
        http::Version::HTTP_10 => Some("HTTP/1.0"),
        http::Version::HTTP_11 => Some("HTTP/1.1"),
        http::Version::HTTP_2 => Some("HTTP/2.0"),
        http::Version::HTTP_3 => Some("HTTP/3.0"),
        _ => None,
    }
}

fn get_header_value_string<'a>(
    headers: &'a HeaderMap,
    header_name: &HeaderName,
) -> Option<&'a str> {
    headers.get(header_name)?.to_str().ok()
}

impl<B> tower_http::trace::OnRequest<B> for OnRequest
where
    B: Body,
{
    fn on_request(&mut self, _request: &http::Request<B>, _span: &tracing::Span) {
        tracing::trace!("started proccessing request");
    }
}

impl<B> tower_http::trace::OnResponse<B> for OnResponse
where
    B: Body,
{
    fn on_response(self, response: &http::Response<B>, latency: Duration, span: &tracing::Span) {
        if let Some(size) = response.body().size_hint().exact() {
            span.record("__http_request.response_size", &size);
        }

        span.record("__http_request.status", &response.status().as_u16());
        span.record("__http_request.latency.seconds", &latency.as_secs());
        span.record("__http_request.latency.nanos", &latency.subsec_nanos());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnFailure;

impl trace::OnFailure<ServerErrorsFailureClass> for OnFailure {
    fn on_failure(
        &mut self,
        failure_classification: ServerErrorsFailureClass,
        latency: Duration,
        span: &tracing::Span,
    ) {
        match failure_classification {
            ServerErrorsFailureClass::StatusCode(status) => {
                span.record("__http_request.status", &status.as_u16());
            }
            ServerErrorsFailureClass::Error(error) => {
                span.record("__http_request.status", &error);
            }
        }

        span.record("__http_request.latency.seconds", &latency.as_secs());
        span.record("__http_request.latency.nanos", &latency.subsec_nanos());
    }
}
