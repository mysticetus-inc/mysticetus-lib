use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use timestamp::Timestamp;
use tracing::field::Visit;
use tracing::span::Id;
use tracing_subscriber::fmt::{FmtContext, FormatFields};
use tracing_subscriber::registry::LookupSpan;

use crate::Stage;
use crate::http_request::{RequestTrace, TraceHeader};

pub const ALERT_ERROR_NAME: &str = "@type";
pub const ALERT_ERROR_VALUE: &str =
    "type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent";

pub const TRACE_KEY: &str = "logging.googleapis.com/trace";
pub const TIMESTAMP_KEY: &str = "timestamp";
pub const SEVERITY_KEY: &str = "severity";
pub const SPAN_ID_KEY: &str = "logging.googleapis.com/spanId";
pub const HTTP_REQUEST_KEY: &str = "httpRequest";
pub const LABELS_KEY: &str = "logging.googleapis.com/labels";
pub const SOURCE_LOCATION_KEY: &str = "logging.googleapis.com/sourceLocation";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation<'a> {
    pub file: Option<&'a str>,
    pub line: u32,
    pub function: &'a str,
}

impl<'a> SourceLocation<'a> {
    pub(crate) fn new(meta: &'a tracing::Metadata<'a>) -> Self {
        Self {
            file: meta.file(),
            line: meta.line().unwrap_or_default(),
            function: meta.target(),
        }
    }
}

fn serialize_span_id<M>(span_id: &Id, map: &mut M) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    let span_id_bytes = span_id.into_u64().to_be_bytes();

    let mut dst = [0_u8; 2 * std::mem::size_of::<u64>()];

    hex::encode_to_slice(span_id_bytes, &mut dst).expect("dst has enough space to encode 8 bytes");

    let hex_str =
        std::str::from_utf8(&dst).expect("successful hex encoding should always be valid ascii");

    map.serialize_entry(SPAN_ID_KEY, hex_str)
}

pub fn get_severity_string(level: &tracing::Level) -> &'static str {
    // ordering is backwards from (at least my) intuition,
    // and is based on the amount of "verbosity"
    // i.e.
    // trace > info, because trace is more verbose than info.
    // error < info, because info is more verbose
    if level >= &tracing::Level::TRACE {
        "DEBUG"
    } else if level >= &tracing::Level::DEBUG {
        "INFO"
    } else if level >= &tracing::Level::INFO {
        "NOTICE"
    } else if level >= &tracing::Level::WARN {
        "WARN"
    } else {
        "ERROR"
    }
}
