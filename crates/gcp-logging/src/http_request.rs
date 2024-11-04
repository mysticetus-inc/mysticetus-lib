use std::fmt;
use std::num::{FpCategory, NonZeroU64};

use http::{HeaderName, HeaderValue, Method, StatusCode, Uri, Version, header};
use http_body::Body;
use serde::ser::SerializeMap;
use timestamp::Duration;
use tracing::field::Visit;

pub const TRACE_CTX_HEADER: HeaderName = HeaderName::from_static("x-cloud-trace-context");

pub mod fields {
    pub const HTTP_REQUEST_PREFIX: &str = "httpRequest.";

    pub const HTTP_REQUEST_PROTOCOL: &str = "httpRequest.protocol";
    pub const HTTP_REQUEST_METHOD: &str = "httpRequest.requestMethod";
    pub const HTTP_REQUEST_URL: &str = "httpRequest.requestUrl";
    pub const HTTP_REQUEST_SIZE: &str = "httpRequest.requestSize";
    pub const HTTP_REQUEST_USER_AGENT: &str = "httpRequest.userAgent";
    pub const HTTP_REQUEST_REFERER: &str = "httpRequest.referer";
    pub const HTTP_REQUEST_STATUS: &str = "httpRequest.status";
    pub const HTTP_REQUEST_RESPONSE_SIZE: &str = "httpRequest.responseSize";
    pub const HTTP_REQUEST_LATENCY_SECONDS: &str = "httpRequest.latency.seconds";
    pub const HTTP_REQUEST_LATENCY_NANOS: &str = "httpRequest.latency.nanos";
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    #[serde(serialize_with = "serialize::protocol")]
    protocol: Version,
    #[serde(serialize_with = "serialize::method")]
    request_method: Method,
    request_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_size: Option<u64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    response_size: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize::opt_latency"
    )]
    latency: Option<Duration>,
}

impl HttpRequest {
    pub fn from_request<B: Body>(request: &http::Request<B>) -> Self {
        Self {
            protocol: request.version(),
            request_method: request.method().clone(),
            request_url: request.uri().to_string(),
            request_size: request.body().size_hint().exact(),
            referer: request.headers().get(header::REFERER).cloned(),
            user_agent: request.headers().get(header::USER_AGENT).cloned(),
            status: None,
            response_size: None,
            latency: None,
        }
    }

    pub fn update_response(
        &mut self,
        latency: Duration,
        status: StatusCode,
        response_size: Option<u64>,
    ) {
        self.status = Some(status);
        self.latency = Some(latency);
        self.response_size = response_size;
    }
}

pub struct HttpRequestPayload<'a> {
    event: &'a tracing::Event<'a>,
}

impl<'a> HttpRequestPayload<'a> {
    pub fn new(event: &'a tracing::Event<'a>) -> Self {
        Self { event }
    }
}

impl<'a> serde::Serialize for HttpRequestPayload<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_ser = serializer.serialize_map(None)?;

        let mut visitor = HttpRequestVisitor::new(&mut map_ser);

        self.event.record(&mut visitor);

        visitor.finish()?;

        map_ser.end()
    }
}

enum HttpField {
    Normal(&'static str),
    LatencyNanos,
    LatencySeconds,
}

impl HttpField {
    pub fn from_field(field: &tracing::field::Field) -> Option<Self> {
        let name = field.name();
        let name = name.strip_prefix(fields::HTTP_REQUEST_PREFIX)?;

        match name {
            "latency.seconds" => Some(Self::LatencySeconds),
            "latency.nanos" => Some(Self::LatencyNanos),
            _ => Some(Self::Normal(name)),
        }
    }
}

struct HttpRequestVisitor<'a, M: SerializeMap> {
    latency_seconds: Option<Int>,
    latency_nanos: Option<Int>,
    latency_written: bool,
    http_fields: usize,
    map_ser: &'a mut M,
    error: Option<M::Error>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Int {
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
}

impl Int {
    fn try_from_str(mut s: &str) -> Option<Self> {
        s = s.trim();

        macro_rules! try_parse_as_type {
            ($variant:ident) => {{
                if let Ok(int) = s.parse() {
                    return Some(Self::$variant(int));
                }
            }};
        }

        if s.starts_with('-') {
            try_parse_as_type!(I64);
            try_parse_as_type!(I128);
        } else {
            try_parse_as_type!(U64);
            try_parse_as_type!(U128);
        }

        None
    }

    fn try_from_f64(mut f: f64) -> Option<Self> {
        // helper macro to ensure that casts dont truncate
        macro_rules! within_bounds {
            ($value:expr => $t:ty) => {{
                const MIN: f64 = <$t>::MIN as f64;
                const MAX: f64 = <$t>::MAX as f64;

                (MIN..=MAX).contains(&$value)
            }};
        }

        match f.classify() {
            FpCategory::Infinite | FpCategory::Nan => return None,
            FpCategory::Subnormal | FpCategory::Zero => return Some(Self::U64(0)),
            FpCategory::Normal => (),
        }

        f = f.round();

        if f.is_sign_negative() {
            if within_bounds!(f => i64) {
                return Some(Self::I64(f as i64));
            }

            if within_bounds!(f => i128) {
                return Some(Self::I128(f as i128));
            }
        } else {
            if within_bounds!(f => u64) {
                return Some(Self::U64(f as u64));
            }

            if within_bounds!(f => u128) {
                return Some(Self::U128(f as u128));
            }
        }

        None
    }
}

impl serde::Serialize for Int {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::I64(i) => serializer.serialize_i64(i),
            Self::U64(u) => serializer.serialize_u64(u),
            Self::I128(i) => serializer.serialize_i128(i),
            Self::U128(u) => serializer.serialize_u128(u),
        }
    }
}

impl<'a, M: SerializeMap> HttpRequestVisitor<'a, M> {
    pub(super) fn new(map_ser: &'a mut M) -> Self {
        Self {
            map_ser,
            latency_seconds: None,
            http_fields: 0,
            latency_nanos: None,
            latency_written: false,
            error: None,
        }
    }

    pub(super) fn finish(self) -> Result<usize, M::Error> {
        match self.error {
            None => Ok(self.http_fields),
            Some(error) => Err(error),
        }
    }

    fn try_insert_latency_nanos(&mut self, f: impl FnOnce(&mut Option<Int>)) {
        if self.latency_nanos.is_none() {
            f(&mut self.latency_nanos);

            if self.latency_nanos.is_some() {
                self.http_fields += 1;
                self.try_serialize_latency();
            }
        }
    }

    fn try_insert_latency_seconds(&mut self, f: impl FnOnce(&mut Option<Int>)) {
        if self.latency_seconds.is_none() {
            f(&mut self.latency_seconds);

            if self.latency_seconds.is_some() {
                self.http_fields += 1;
                self.try_serialize_latency();
            }
        }
    }

    fn try_serialize_latency(&mut self) {
        #[derive(Debug, Clone, Copy, serde::Serialize)]
        struct LatencyRepr<'a> {
            seconds: &'a Int,
            nanos: &'a Int,
        }

        if self.latency_written {
            return;
        }

        match (self.latency_seconds, self.latency_nanos) {
            (Some(ref seconds), Some(ref nanos)) => {
                match self
                    .map_ser
                    .serialize_entry("latency", &LatencyRepr { seconds, nanos })
                {
                    Ok(_) => self.latency_written = true,
                    Err(error) => self.error = Some(error),
                }
            }
            (_, _) => (),
        }
    }

    fn try_serialize<S: serde::Serialize>(&mut self, key: &str, value: S) {
        match self.map_ser.serialize_entry(key, &value) {
            Ok(_) => self.http_fields += 1,
            Err(error) => self.error = Some(error),
        }
    }
}

macro_rules! impl_visit_fn {
    ($self:expr; $field:expr => $value:expr $(;)?) => {
        impl_visit_fn!($self; $field => $value; seconds: (), nanos: ())
    };
    /*
    (
        $self:expr; $field:expr => $value:expr;
        seconds: $seconds_blk:block $(,)?
    ) => {
        impl_visit_fn!($self; $field => $value; seconds: $seconds_blk, nanos: {})
    };
    (
        $self:expr; $field:expr => $value:expr;
        nanos: $nanos_blk:block $(,)?
    ) => {
        impl_visit_fn!($self; $field => $value; seconds: {}, nanos: $nanos_blk)
    };
    */
    (
        $self:expr; $field:expr => $value:expr;
        seconds: $seconds_blk:expr,
        nanos: $nanos_blk:expr $(,)?
    ) => {{
        if $self.error.is_some() {
            return;
        }

        match HttpField::from_field($field) {
            Some(HttpField::Normal(name)) => $self.try_serialize(name, $value),
            Some(HttpField::LatencyNanos) => $nanos_blk,
            Some(HttpField::LatencySeconds) => $seconds_blk,
            None => (),
        }
    }};
}

impl<'a, S> Visit for HttpRequestVisitor<'a, S>
where
    S: SerializeMap,
{
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        fn try_parse_int_debug(dst: &mut Option<Int>, value: &dyn std::fmt::Debug) {
            let int_opt = if let Some(str_repr) = format_args!("{:?}", value).as_str() {
                Int::try_from_str(str_repr)
            } else {
                crate::utils::TlsStringBuf::with_buf(|buf| {
                    fmt::write(buf, format_args!("{value:?}"))
                        .expect("string formatting should never fail");
                    Int::try_from_str(&buf)
                })
            };

            if let Some(int) = int_opt {
                *dst = Some(int);
            }
        }

        impl_visit_fn! {
            self; field => crate::utils::SerializeDebug(value);
            seconds: self.try_insert_latency_seconds(|dst| try_parse_int_debug(dst, value)),
            nanos: self.try_insert_latency_nanos(|dst| try_parse_int_debug(dst, value)),
        }
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        fn set_if_int(dst: &mut Option<Int>, value: f64) {
            if let Some(int) = Int::try_from_f64(value) {
                *dst = Some(int);
            }
        }

        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| set_if_int(dst, value)),
            nanos: self.try_insert_latency_nanos(|dst| set_if_int(dst, value)),
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| *dst = Some(Int::I64(value))),
            nanos: self.try_insert_latency_nanos(|dst| *dst = Some(Int::I64(value))),
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| *dst = Some(Int::U64(value))),
            nanos: self.try_insert_latency_nanos(|dst| *dst = Some(Int::U64(value))),
        }
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| *dst = Some(Int::I128(value))),
            nanos: self.try_insert_latency_nanos(|dst| *dst = Some(Int::I128(value))),
        }
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| *dst = Some(Int::U128(value))),
            nanos: self.try_insert_latency_nanos(|dst| *dst = Some(Int::U128(value))),
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        impl_visit_fn!(self; field => value);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        fn try_parse_int(dst: &mut Option<Int>, s: &str) {
            if let Some(int) = Int::try_from_str(s) {
                *dst = Some(int);
            }
        }

        impl_visit_fn! {
            self; field => value;
            seconds: self.try_insert_latency_seconds(|dst| try_parse_int(dst, value)),
            nanos: self.try_insert_latency_nanos(|dst| try_parse_int(dst, value)),
        }
    }

    fn record_error(
        &mut self,
        _field: &tracing::field::Field,
        _value: &(dyn std::error::Error + 'static),
    ) {
        // the fields for HttpRequest should never be an error, so we can always skip this call
    }
}

mod serialize {
    use http::{HeaderValue, Method, StatusCode, Uri, Version};
    use timestamp::Duration;

    #[inline]
    pub(super) fn protocol<S>(vers: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *vers {
            Version::HTTP_09 => serializer.serialize_str("HTTP/0.9"),
            Version::HTTP_10 => serializer.serialize_str("HTTP/1.0"),
            Version::HTTP_11 => serializer.serialize_str("HTTP/1.1"),
            Version::HTTP_2 => serializer.serialize_str("HTTP/2"),
            Version::HTTP_3 => serializer.serialize_str("HTTP/3"),
            _ => serializer.collect_str(&format_args!("{vers:?}")),
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
            Some(ref header) => serializer.serialize_some(&SerializeHeader(header)),
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
