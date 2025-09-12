use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Version, header};
use http_body::Body;
pub use size::Size;
use timestamp::Duration;

pub const TRACE_CTX_HEADER: HeaderName = HeaderName::from_static("x-cloud-trace-context");

pub(crate) const RESPONSE_STATUS_KEY: &str = "__resp_status__";
pub(crate) const RESPONSE_SIZE_KEY: &str = "__resp_size__";
pub(crate) const RESPONSE_LATENCY_KEY: &str = "__resp_latency__";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    #[serde(serialize_with = "serialize::protocol")]
    protocol: Version,
    #[serde(serialize_with = "serialize::method")]
    request_method: Method,
    #[serde(serialize_with = "serialize::url")]
    request_url: http::Uri,
    #[serde(skip_serializing_if = "Size::is_unknown")]
    request_size: Size,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize::opt_header"
    )]
    user_agent: Option<HeaderValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize::opt_header"
    )]
    referer: Option<HeaderValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize::opt_status_code"
    )]
    status: Option<StatusCode>,
    #[serde(skip_serializing_if = "Size::is_unknown")]
    response_size: Size,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize::opt_latency"
    )]
    latency: Option<Duration>,
}

impl HttpRequest {
    pub fn from_request<B: Body>(request: &http::Request<B>) -> Self {
        let request_size = request
            .body()
            .size_hint()
            .exact()
            .and_then(Size::new)
            .or_else(|| try_read_content_len(request.headers()))
            .unwrap_or_else(Size::unknown);

        Self {
            protocol: request.version(),
            request_method: request.method().clone(),
            request_url: request.uri().clone(),
            request_size,
            referer: request.headers().get(header::REFERER).cloned(),
            user_agent: request.headers().get(header::USER_AGENT).cloned(),
            status: None,
            response_size: Size::unknown(),
            latency: None,
        }
    }
}

pub(crate) fn get_response_size<B: Body>(response: &http::Response<B>) -> Size {
    response
        .body()
        .size_hint()
        .exact()
        .and_then(Size::new)
        .or_else(|| try_read_content_len(response.headers()))
        .unwrap_or(Size::unknown())
}

fn try_read_content_len(headers: &HeaderMap) -> Option<Size> {
    headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.parse().ok())
        .and_then(Size::new)
}

mod serialize {
    use http::{HeaderValue, Method, StatusCode, Uri, Version};
    use timestamp::Duration;

    #[inline]
    pub(super) fn protocol<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *version {
            Version::HTTP_09 => serializer.serialize_str("HTTP/0.9"),
            Version::HTTP_10 => serializer.serialize_str("HTTP/1.0"),
            Version::HTTP_11 => serializer.serialize_str("HTTP/1.1"),
            Version::HTTP_2 => serializer.serialize_str("HTTP/2"),
            Version::HTTP_3 => serializer.serialize_str("HTTP/3"),
            // this branch should never hit, at least until http/4 is a thing
            _ => serializer.collect_str(&format_args!("{version:?}")),
        }
    }

    #[inline]
    pub(super) fn method<S>(meth: &Method, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(meth.as_str())
    }

    #[inline]
    pub(super) fn url<S>(url: &Uri, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(url)
    }

    #[inline]
    pub(super) fn opt_status_code<S>(
        status: &Option<StatusCode>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match status {
            Some(status) => serializer.serialize_some(&status.as_u16()),
            None => serializer.serialize_none(),
        }
    }

    #[inline]
    pub(super) fn opt_header<S>(opt: &Option<HeaderValue>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match opt {
            None => serializer.serialize_none(),
            Some(header) => serializer.serialize_some(&SerializeHeader(header)),
        }
    }

    pub struct SerializeHeader<'a>(pub &'a HeaderValue);

    impl serde::Serialize for SerializeHeader<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self.0.to_str() {
                Ok(header_str) => serializer.serialize_str(header_str),
                Err(_) => serializer.collect_str(&self.0.as_bytes().escape_ascii()),
            }
        }
    }

    #[inline]
    pub(super) fn opt_latency<S>(
        latency: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match latency {
            None => serializer.serialize_none(),
            Some(latency) => serializer.serialize_some(&SerializeDurationAsProto(*latency)),
        }
    }

    pub struct SerializeDurationAsProto(Duration);

    impl serde::Serialize for SerializeDurationAsProto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeMap;

            let mut map = serializer.serialize_map(Some(2))?;

            map.serialize_entry("seconds", &self.0.whole_seconds())?;
            map.serialize_entry("nanos", &self.0.subsec_nanoseconds())?;

            map.end()
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestTrace {
    pub trace_header: Option<HeaderValue>,
    pub request: HttpRequest,
}

impl RequestTrace {
    pub fn new<B: Body>(request: &http::Request<B>) -> Self {
        Self {
            trace_header: request.headers().get(TRACE_CTX_HEADER).cloned(),
            request: HttpRequest::from_request(&request),
        }
    }
}

/// Helper type to format the logging trace in the correct format,
/// without allocating
pub struct TraceHeader<'a> {
    project_id: &'static str,
    header: &'a HeaderValue,
}

impl<'a> TraceHeader<'a> {
    #[inline]
    pub fn new(project_id: &'static str, header: &'a HeaderValue) -> Self {
        Self { project_id, header }
    }
}

impl std::fmt::Display for TraceHeader<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { project_id, header } = self;

        let trace_bytes = header.as_bytes();

        let trace = match memchr::memchr(b'/', trace_bytes) {
            Some(end_index) => &trace_bytes[..end_index],
            None => trace_bytes,
        };

        if trace.is_ascii() {
            // SAFETY: we just checked it was valid ascii. This should only ever be ascii and
            // not utf8, so we only check for ascii to avoid the extra overhead from
            // checking utf8
            let trace_str = unsafe { std::str::from_utf8_unchecked(trace) };
            write!(f, "projects/{project_id}/traces/{trace_str}")
        } else {
            // if the trace isn't ascii, something is probably wrong,
            // but write it anyways in the hopes google can interpret it
            let trace_escaped = trace.escape_ascii();
            write!(f, "projects/{project_id}/traces/{trace_escaped}")
        }
    }
}

impl serde::Serialize for TraceHeader<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

pub mod size {
    /// A newtype for request/response sizes, with a niche value
    /// of [`u64::MAX`] indicating the value is unknown. Given no API
    /// would ever accept/respond with a single payload of 16 million TiB,
    /// we can safely assume that we'll never encounter that [`u64::MAX`] in
    /// the real world, and worst case if we get an incorrect content len,
    /// we'll just skip logging the value.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct Size(u64);

    impl std::fmt::Debug for Size {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Size").field(&self.get()).finish()
        }
    }

    impl std::fmt::Display for Size {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.get() {
                None => f.write_str("unknown size"),
                Some(bytes) => {
                    let mut prefix = 0;
                    let mut remainder = bytes;
                    while remainder > 1024 {
                        remainder /= 1024;
                        prefix += 1;
                    }

                    match prefix {
                        1 => write!(f, "{remainder} KiB"),
                        2 => write!(f, "{remainder} MiB"),
                        3 => write!(f, "{remainder} GiB"),
                        4 => write!(f, "{remainder} TiB"),
                        _ => write!(f, "{bytes} bytes"),
                    }
                }
            }
        }
    }

    impl Default for Size {
        fn default() -> Self {
            Self::unknown()
        }
    }

    impl serde::Serialize for Size {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.get().serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Size {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let optional: Option<u64> = serde::Deserialize::deserialize(deserializer)?;
            Ok(optional.and_then(Size::new).unwrap_or(Size::unknown()))
        }
    }

    impl Size {
        #[inline]
        pub const fn unknown() -> Self {
            Self(u64::MAX)
        }

        #[inline]
        pub const fn new(size: u64) -> Option<Self> {
            if size == u64::MAX {
                None
            } else {
                Some(Self(size))
            }
        }

        #[inline]
        pub fn get(&self) -> Option<u64> {
            match self.0 {
                u64::MAX => None,
                other => Some(other),
            }
        }

        #[inline]
        pub fn is_unknown(&self) -> bool {
            self.0 == u64::MAX
        }
    }
}
