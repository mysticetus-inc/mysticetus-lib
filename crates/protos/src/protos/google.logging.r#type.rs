// This file is @generated by prost-build.
/// A common proto for logging HTTP requests. Only contains semantics
/// defined by the HTTP specification. Product-specific logging
/// information MUST be defined in a separate message.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpRequest {
    /// The request method. Examples: `"GET"`, `"HEAD"`, `"PUT"`, `"POST"`.
    #[prost(string, tag = "1")]
    pub request_method: ::prost::alloc::string::String,
    /// The scheme (http, https), the host name, the path and the query
    /// portion of the URL that was requested.
    /// Example: `"<http://example.com/some/info?color=red"`.>
    #[prost(string, tag = "2")]
    pub request_url: ::prost::alloc::string::String,
    /// The size of the HTTP request message in bytes, including the request
    /// headers and the request body.
    #[prost(int64, tag = "3")]
    pub request_size: i64,
    /// The response code indicating the status of response.
    /// Examples: 200, 404.
    #[prost(int32, tag = "4")]
    pub status: i32,
    /// The size of the HTTP response message sent back to the client, in bytes,
    /// including the response headers and the response body.
    #[prost(int64, tag = "5")]
    pub response_size: i64,
    /// The user agent sent by the client. Example:
    /// `"Mozilla/4.0 (compatible; MSIE 6.0; Windows 98; Q312461; .NET
    /// CLR 1.0.3705)"`.
    #[prost(string, tag = "6")]
    pub user_agent: ::prost::alloc::string::String,
    /// The IP address (IPv4 or IPv6) of the client that issued the HTTP
    /// request. This field can include port information. Examples:
    /// `"192.168.1.1"`, `"10.0.0.1:80"`, `"FE80::0202:B3FF:FE1E:8329"`.
    #[prost(string, tag = "7")]
    pub remote_ip: ::prost::alloc::string::String,
    /// The IP address (IPv4 or IPv6) of the origin server that the request was
    /// sent to. This field can include port information. Examples:
    /// `"192.168.1.1"`, `"10.0.0.1:80"`, `"FE80::0202:B3FF:FE1E:8329"`.
    #[prost(string, tag = "13")]
    pub server_ip: ::prost::alloc::string::String,
    /// The referer URL of the request, as defined in
    /// [HTTP/1.1 Header Field
    /// Definitions](<https://datatracker.ietf.org/doc/html/rfc2616#section-14.36>).
    #[prost(string, tag = "8")]
    pub referer: ::prost::alloc::string::String,
    /// The request processing latency on the server, from the time the request was
    /// received until the response was sent.
    #[prost(message, optional, tag = "14")]
    pub latency: ::core::option::Option<super::super::protobuf::Duration>,
    /// Whether or not a cache lookup was attempted.
    #[prost(bool, tag = "11")]
    pub cache_lookup: bool,
    /// Whether or not an entity was served from cache
    /// (with or without validation).
    #[prost(bool, tag = "9")]
    pub cache_hit: bool,
    /// Whether or not the response was validated with the origin server before
    /// being served from cache. This field is only meaningful if `cache_hit` is
    /// True.
    #[prost(bool, tag = "10")]
    pub cache_validated_with_origin_server: bool,
    /// The number of HTTP response bytes inserted into cache. Set only when a
    /// cache fill was attempted.
    #[prost(int64, tag = "12")]
    pub cache_fill_bytes: i64,
    /// Protocol used for the request. Examples: "HTTP/1.1", "HTTP/2", "websocket"
    #[prost(string, tag = "15")]
    pub protocol: ::prost::alloc::string::String,
}
/// The severity of the event described in a log entry, expressed as one of the
/// standard severity levels listed below.  For your reference, the levels are
/// assigned the listed numeric values. The effect of using numeric values other
/// than those listed is undefined.
///
/// You can filter for log entries by severity.  For example, the following
/// filter expression will match log entries with severities `INFO`, `NOTICE`,
/// and `WARNING`:
///
///      severity > DEBUG AND severity <= WARNING
///
/// If you are writing log entries, you should map other severity encodings to
/// one of these standard levels. For example, you might map all of Java's FINE,
/// FINER, and FINEST levels to `LogSeverity.DEBUG`. You can preserve the
/// original severity level in the log entry payload if you wish.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LogSeverity {
    /// (0) The log entry has no assigned severity level.
    Default = 0,
    /// (100) Debug or trace information.
    Debug = 100,
    /// (200) Routine information, such as ongoing status or performance.
    Info = 200,
    /// (300) Normal but significant events, such as start up, shut down, or
    /// a configuration change.
    Notice = 300,
    /// (400) Warning events might cause problems.
    Warning = 400,
    /// (500) Error events are likely to cause problems.
    Error = 500,
    /// (600) Critical events cause more severe problems or outages.
    Critical = 600,
    /// (700) A person must take an action immediately.
    Alert = 700,
    /// (800) One or more systems are unusable.
    Emergency = 800,
}
impl LogSeverity {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Notice => "NOTICE",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
            Self::Alert => "ALERT",
            Self::Emergency => "EMERGENCY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DEFAULT" => Some(Self::Default),
            "DEBUG" => Some(Self::Debug),
            "INFO" => Some(Self::Info),
            "NOTICE" => Some(Self::Notice),
            "WARNING" => Some(Self::Warning),
            "ERROR" => Some(Self::Error),
            "CRITICAL" => Some(Self::Critical),
            "ALERT" => Some(Self::Alert),
            "EMERGENCY" => Some(Self::Emergency),
            _ => None,
        }
    }
}
