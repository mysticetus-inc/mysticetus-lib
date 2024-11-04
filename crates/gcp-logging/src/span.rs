use std::fmt;

use http::{HeaderMap, HeaderName, HeaderValue};
use http_body::Body;

use crate::http_request::fields::*;
use crate::http_request::{HttpRequest, TRACE_CTX_HEADER};
use crate::subscriber::{RequestTrace, SubscriberHandle};

pub const SPAN_FIELD_NAME: &str = "logging.googleapis.com/spanId";
pub const TRACE_FIELD_NAME: &str = "logging.googleapis.com/trace";

fn get_header_str(headers: &HeaderMap, name: HeaderName) -> Option<&str> {
    headers.get(name).and_then(|h| h.to_str().ok())
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

pub(super) fn make_span2<B: Body>(
    request: &http::Request<B>,
    handle: &SubscriberHandle,
) -> tracing::Span {
    let span = tracing::info_span!("request");

    if let Some(span_id) = span.id() {
        handle.insert_new_trace(RequestTrace::new(span_id, request));
    }

    span
}

pub(super) fn make_span<B>(
    request: &mut http::Request<B>,
    handle: &SubscriberHandle,
) -> (tracing::Span, RequestTrace) {
    let trace_header = request.headers().get(TRACE_CTX_HEADER).cloned();

    let headers = request.headers();

    let trace = trace_header.is_some().then_some(tracing::field::Empty);

    let proto = get_version_str(request.version());
    let request_size = get_header_str(headers, http::header::CONTENT_LENGTH);
    let user_agent = get_header_str(headers, http::header::USER_AGENT);
    let referer = get_header_str(headers, http::header::REFERER);

    /// this macro effectively does the following, but it does __not__ include the field if the
    /// value is None.
    ///
    /// ```rust
    /// tracing::info_span!(
    ///    "request",
    ///    // variable fields
    ///    {TRACE_FIELD_NAME} = get_header_str(headers, TRACE_CTX_HEADER),
    ///    {HTTP_REQUEST_PROTOCOL} = get_version_str(request.version()),
    ///    {HTTP_REQUEST_SIZE} = get_header_str(headers, http::header::CONTENT_LENGTH),
    ///    {HTTP_REQUEST_USER_AGENT} = get_header_str(headers, http::header::USER_AGENT),
    ///    {HTTP_REQUEST_REFERER} = get_header_str(headers, http::header::REFERER),
    ///    // these are always known / defined ahead of time
    ///    {HTTP_REQUEST_METHOD} = request.method().as_str(),
    ///    {HTTP_REQUEST_URL} = %request.uri(),
    ///    {HTTP_REQUEST_STATUS} = tracing::field::Empty,
    ///    {HTTP_REQUEST_RESPONSE_SIZE} = tracing::field::Empty,
    ///    {HTTP_REQUEST_LATENCY_SECONDS} = tracing::field::Empty,
    ///    {HTTP_REQUEST_LATENCY_NANOS} = tracing::field::Empty,
    /// );
    /// ```
    macro_rules! make_span {
        ($($name:ident : $val:ident),* $(,)?) => {
            make_span!(terms = [], rest = [$($name: $val,)*])
        };
        (terms = [$($t:tt)*], rest = [$name:ident : $val:ident, $($rest:tt)*]) => {
            match $val {
                Some($val) => make_span!(terms = [$($t)* {$name} = $val,], rest = [$($rest)*]),
                None => make_span!(terms = [$($t)*], rest = [$($rest)*]),
            }
        };
        (terms = [$($t:tt)*], rest = [$(,)?]) => {
            tracing::info_span!(
                "request",
                {HTTP_REQUEST_METHOD} = request.method().as_str(),
                {HTTP_REQUEST_URL} = %request.uri(),
                {HTTP_REQUEST_STATUS} = tracing::field::Empty,
                {HTTP_REQUEST_RESPONSE_SIZE} = tracing::field::Empty,
                {HTTP_REQUEST_LATENCY_SECONDS} = tracing::field::Empty,
                {HTTP_REQUEST_LATENCY_NANOS} = tracing::field::Empty,
                $($t)*
            )
        };
    }

    let span = make_span! {
        TRACE_FIELD_NAME: trace,
        HTTP_REQUEST_PROTOCOL: proto,
        HTTP_REQUEST_SIZE: request_size,
        HTTP_REQUEST_USER_AGENT: user_agent,
        HTTP_REQUEST_REFERER: referer,
    };

    // (span, trace_header)

    todo!()
}

mod trace {
    use std::fmt;

    const PROJECTS: &str = "projects/";
    const TRACES: &str = "/traces/";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Trace<'a> {
        pub project_id: &'a str,
        pub raw_trace: &'a str,
    }

    impl Trace<'_> {
        #[inline]
        fn parts(&self) -> (&str, &str) {
            let trace = self.raw_trace.split('/').next().unwrap_or(self.raw_trace);
            (self.project_id, trace)
        }

        pub fn alloc_to(&self, dst: &mut String) {
            let (project_id, trace) = self.parts();
            dst.clear();
            dst.reserve(needed_capacity(project_id, trace));
            insert(project_id, trace, dst);
        }

        pub fn alloc(&self) -> String {
            let (project_id, trace) = self.parts();
            let mut dst = String::with_capacity(needed_capacity(project_id, trace));
            insert(project_id, trace, &mut dst);
            dst
        }
    }

    impl fmt::Display for Trace<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let (project_id, trace) = self.parts();
            write!(f, "{PROJECTS}{project_id}{TRACES}{trace}")
        }
    }

    #[inline]
    const fn needed_capacity(project_id: &str, trace: &str) -> usize {
        PROJECTS.len() + project_id.len() + TRACES.len() + trace.len()
    }

    fn insert(project_id: &str, trace: &str, dst: &mut String) {
        debug_assert!(dst.capacity() >= needed_capacity(project_id, trace));

        dst.push_str(PROJECTS);
        dst.push_str(project_id);
        dst.push_str(TRACES);
        dst.push_str(trace);
    }
}
