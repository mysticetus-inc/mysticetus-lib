use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use timestamp::Timestamp;
use tracing::span::Id;
use tracing_subscriber::fmt::{FmtContext, FormatFields};
use tracing_subscriber::registry::LookupSpan;

use crate::span::SPAN_FIELD_NAME;
use crate::subscriber::RequestTrace;
use crate::trace_layer::{ActiveTraces, TraceHeader};
use crate::Stage;

const ALERT_ERROR_NAME: &str = "@type";
const ALERT_ERROR_VALUE: &str =
    "type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent";

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

pub struct LogEntry<'a, L, N, O = crate::DefaultLogOptions> {
    pub project_id: &'a str,
    pub timestamp: Option<Timestamp>,
    pub traces: &'a ActiveTraces,
    pub ctx: &'a FmtContext<'a, L, N>,
    pub stage: Stage,
    pub options: &'a O,
    pub event: &'a tracing::Event<'a>,
}

impl<'a, L, N, O> LogEntry<'a, L, N, O>
where
    L: tracing::Subscriber,
    for<'b> L: LookupSpan<'b>,
    for<'b> N: FormatFields<'b> + 'static,
    O: crate::LogOptions,
{
    pub fn new(
        project_id: &'a str,
        ctx: &'a FmtContext<'a, L, N>,
        stage: Stage,
        options: &'a O,
        traces: &'a ActiveTraces,
        event: &'a tracing::Event<'a>,
    ) -> Self {
        Self {
            project_id,
            timestamp: None,
            traces,
            stage,
            ctx,
            options,
            event,
        }
    }

    fn serialize_request_trace<M>(&self, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap,
    {
        self.with_request_trace::<_, M::Error>(|art| {
            if self.options.include_http_info(self.event.metadata()) {
                map.serialize_entry("httpRequest", &art.request_trace.request)?;
            }

            if let Some(ref header) = art.request_trace.trace_header {
                map.serialize_entry(
                    crate::span::TRACE_FIELD_NAME,
                    &TraceHeader::new(self.project_id, header),
                )?;
                serialize_span_id(art.current_span_id, map)?;
            }

            Ok(())
        })
    }

    fn with_request_trace<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce(ActiveRequestTrace<'_>) -> Result<(), E>,
    {
        if let Some(scope) = self.ctx.event_scope() {
            let mut current_span_id = None;

            for refer in scope {
                let id = refer.id();
                // downcast since we don't want to be able to mutate it
                let current_span_id: &Id = current_span_id.get_or_insert_with(|| refer.id());

                if let Some(request_trace) = self.traces.get(&id) {
                    return f(ActiveRequestTrace {
                        request_trace,
                        trace_span_id: &id,
                        current_span_id,
                    });
                }
            }
        }

        Ok(())
    }
}

pub struct ActiveRequestTrace<'a> {
    request_trace: dashmap::mapref::one::Ref<'a, Id, RequestTrace, fxhash::FxBuildHasher>,
    trace_span_id: &'a Id,
    current_span_id: &'a Id,
}

macro_rules! serialize_with {
    ($value:expr; $t:ident:: $f:ident) => {{
        struct SerializeWith<'a>(&'a $t);

        impl serde::Serialize for SerializeWith<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $t::$f(&self.0, serializer)
            }
        }

        &SerializeWith(&$value)
    }};
    ($value:expr; $t:ty => $f:ident) => {{
        struct SerializeWith<'a>(&'a $t);

        impl serde::Serialize for SerializeWith<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $f(&self.0, serializer)
            }
        }

        &SerializeWith(&$value)
    }};
}
impl<'a, L, N, O> serde::Serialize for LogEntry<'a, L, N, O>
where
    L: tracing::Subscriber,
    for<'b> L: LookupSpan<'b>,
    for<'b> N: FormatFields<'b> + 'static,
    O: crate::LogOptions,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        let meta = self.event.metadata();

        let timestamp = self.timestamp.unwrap_or_else(Timestamp::now);

        map.serialize_entry(
            "timestamp",
            serialize_with!(timestamp; Timestamp::serialize_as_proto),
        )?;
        map.serialize_entry(
            "severity",
            serialize_with!(meta.level(); tracing::Level => serialize_as_severity),
        )?;
        map.serialize_entry(
            "logging.googleapis.com/sourceLocation",
            &SourceLocation::new(meta),
        )?;

        if self.options.include_stage(self.stage, meta) {
            map.serialize_entry("stage", &self.stage)?;
        }

        self.serialize_request_trace(&mut map)?;

        let mut visitor = crate::payload::PayloadVisitor::new(meta, &mut map, self.options);

        self.event.record(&mut visitor);

        let should_alert = visitor.finish()?;

        /*
        if matches!(should_alert, AlertFound::Yes) || self.options.treat_as_error(meta) {
            map.serialize_entry(ALERT_ERROR_NAME, ALERT_ERROR_VALUE)?;
        }
        */

        map.end()
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

    map.serialize_entry(SPAN_FIELD_NAME, hex_str)
}

fn get_severity_string(level: &tracing::Level) -> &'static str {
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

fn serialize_as_severity<S>(level: &tracing::Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(get_severity_string(level))
}
