use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use timestamp::Timestamp;
use tracing::field::Visit;
use tracing::span::Id;
use tracing_subscriber::fmt::{FmtContext, FormatFields};
use tracing_subscriber::registry::LookupSpan;

use crate::Stage;
use crate::payload::{EventInfo, serialize_event_payload};
use crate::subscriber::RequestTrace;
use crate::trace_layer::{ActiveTraces, TraceHeader};

const ALERT_ERROR_NAME: &str = "@type";
const ALERT_ERROR_VALUE: &str =
    "type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent";

pub(crate) const LABEL_PREFIX: &str = "label.";

const TRACE_KEY: &str = "logging.googleapis.com/trace";
const TIMESTAMP_KEY: &str = "timestamp";
const SEVERITY_KEY: &str = "severity";
const SPAN_ID_KEY: &str = "logging.googleapis.com/spanId";
const HTTP_REQUEST_KEY: &str = "httpRequest";
const LABELS_KEY: &str = "logging.googleapis.com/labels";
const SOURCE_LOCATION_KEY: &str = "logging.googleapis.com/sourceLocation";

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
                map.serialize_entry(HTTP_REQUEST_KEY, &art.request_trace.request)?;
            }

            if let Some(ref header) = art.request_trace.trace_header {
                map.serialize_entry(TRACE_KEY, &TraceHeader::new(self.project_id, header))?;
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
    #[allow(dead_code)]
    trace_span_id: &'a Id,
    current_span_id: &'a Id,
}

macro_rules! serialize_with {
    ($value:expr; $t:ident:: $f:ident) => {{
        struct SerializeWith<'a>(&'a $t);

        impl serde::Serialize for SerializeWith<'_> {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $t::$f(&self.0, serializer)
            }
        }

        SerializeWith(&$value)
    }};
    ($value:expr; $t:ty => $f:ident) => {{
        struct SerializeWith<'a>(&'a $t);

        impl serde::Serialize for SerializeWith<'_> {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $f(&self.0, serializer)
            }
        }

        SerializeWith(&$value)
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

        if self.options.include_timestamp(self.stage, meta) {
            let timestamp = self.timestamp.unwrap_or_else(Timestamp::now);

            map.serialize_entry(
                TIMESTAMP_KEY,
                &serialize_with!(timestamp; Timestamp::serialize_as_proto),
            )?;
        }

        map.serialize_entry(SEVERITY_KEY, get_severity_string(meta.level()))?;
        map.serialize_entry(SOURCE_LOCATION_KEY, &SourceLocation::new(meta))?;

        self.serialize_request_trace(&mut map)?;

        let EventInfo {
            alert_found,
            labels,
        } = serialize_event_payload(&mut map, self.ctx, self.event, self.options)?;

        if alert_found && self.options.treat_as_error(meta) {
            map.serialize_entry(ALERT_ERROR_NAME, ALERT_ERROR_VALUE)?;
        }

        if labels > 0 {
            map.serialize_entry(
                LABELS_KEY,
                &Labels {
                    labels,
                    stage: self.stage,
                    event: self.event,
                    options: self.options,
                },
            )?;
        } else if self
            .options
            .include_stage(self.stage, self.event.metadata())
        {
            map.serialize_entry(LABELS_KEY, &StageLabel { stage: self.stage })?;
        }

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

    map.serialize_entry(SPAN_ID_KEY, hex_str)
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

#[derive(serde::Serialize)]
struct StageLabel {
    stage: Stage,
}

struct Labels<'a, O> {
    labels: u8,
    stage: Stage,
    event: &'a tracing::Event<'a>,
    options: &'a O,
}

impl<O: crate::LogOptions> serde::Serialize for Labels<'_, O> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let include_stage = self
            .options
            .include_stage(self.stage, self.event.metadata());

        let fields = self.labels as usize + include_stage as usize;

        let mut map = serializer.serialize_map(Some(fields))?;

        let mut visitor = LabelsVisitor {
            map: &mut map,
            error: None,
        };

        self.event.record(&mut visitor);

        if let Some(error) = visitor.error {
            return Err(error);
        }

        if include_stage {
            map.serialize_entry("stage", &self.stage)?;
        }

        map.end()
    }
}

struct LabelsVisitor<'a, M: SerializeMap> {
    map: &'a mut M,
    error: Option<M::Error>,
}

impl<M: SerializeMap> LabelsVisitor<'_, M> {
    #[inline]
    fn serialize_field(
        &mut self,
        field: &tracing::field::Field,
        serialize_entry: impl FnOnce(&mut M, &str) -> Result<(), M::Error>,
    ) {
        if self.error.is_some() {
            return;
        }

        if let Some(field) = field.name().strip_prefix(LABEL_PREFIX) {
            self.error = serialize_entry(self.map, field).err();
        }
    }

    #[cfg_attr(not(feature = "valuable"), allow(dead_code))]
    #[inline]
    fn serialize_field_value(
        &mut self,
        field: &tracing::field::Field,
        value: impl serde::Serialize,
    ) {
        if self.error.is_some() {
            return;
        }

        if let Some(field) = field.name().strip_prefix(LABEL_PREFIX) {
            self.error = self.map.serialize_entry(field, &value).err();
        }
    }
}

macro_rules! record_int_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name(&mut self, field: &tracing::field::Field, value: $arg_ty) {
                self.serialize_field(field, |map, field| {
                    let mut buf = itoa::Buffer::new();
                    map.serialize_entry(field, buf.format(value))
                });
            }
        )*
    };
}

impl<M: SerializeMap> Visit for LabelsVisitor<'_, M> {
    #[cfg(all(tracing_unstable, feature = "valuable"))]
    fn record_value(&mut self, field: &tracing::field::Field, value: valuable::Value<'_>) {
        match value {
            valuable::Value::Bool(b) => self.record_bool(field, b),
            valuable::Value::Char(ch) => {
                let mut buf = [0; 4];
                let s = ch.encode_utf8(&mut buf);
                self.record_str(field, s)
            }
            valuable::Value::F32(float) => self.record_f64(field, float as f64),
            valuable::Value::F64(float) => self.record_f64(field, float),
            valuable::Value::I8(int) => self.record_i64(field, int as i64),
            valuable::Value::I16(int) => self.record_i64(field, int as i64),
            valuable::Value::I32(int) => self.record_i64(field, int as i64),
            valuable::Value::I64(int) => self.record_i64(field, int),
            valuable::Value::I128(int) => self.record_i128(field, int),
            valuable::Value::Isize(int) => self.record_i64(field, int as i64),
            valuable::Value::String(s) => self.record_str(field, s),
            valuable::Value::U8(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U16(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U32(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U64(uint) => self.record_u64(field, uint),
            valuable::Value::U128(uint) => self.record_u128(field, uint),
            valuable::Value::Usize(uint) => self.record_u64(field, uint as u64),
            valuable::Value::Path(path) => match path.to_str() {
                Some(path_str) => self.record_str(field, path_str),
                None => self.serialize_field(field, |map, field| {
                    map.serialize_entry(field, &crate::utils::SerializeDisplay(&path.display()))
                }),
            },
            valuable::Value::Error(error) => self.record_error(field, error),
            valuable::Value::Listable(listable) => todo!(),
            valuable::Value::Mappable(mappable) => todo!(),
            valuable::Value::Structable(structable) => todo!(),
            valuable::Value::Enumerable(enumerable) => todo!(),
            valuable::Value::Tuplable(tuplable) => todo!(),
            valuable::Value::Unit => self.serialize_field_value(field, None::<()>),
            _ => self.record_debug(field, &value),
        }
    }

    #[inline]
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.serialize_field(field, |map, field| {
            map.serialize_entry(field, &crate::utils::SerializeDebug(value))
        });
    }

    record_int_fns! {
        record_i64(i64),
        record_i128(i128),
        record_u64(u64),
        record_u128(u128),
    }

    #[inline]
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.serialize_field(field, |map, field| {
            use std::num::FpCategory;

            let mut buf: ryu::Buffer;

            let float_str = match value.classify() {
                FpCategory::Normal | FpCategory::Subnormal => {
                    buf = ryu::Buffer::new();
                    buf.format_finite(value)
                }
                FpCategory::Zero => "0.0",
                FpCategory::Infinite if value.is_sign_positive() => "Inf",
                FpCategory::Infinite => "-Inf",
                FpCategory::Nan => "NaN",
            };

            map.serialize_entry(field, float_str)
        });
    }

    #[inline]
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.serialize_field(field, |map, field| map.serialize_entry(field, value));
    }

    #[inline]
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.serialize_field(field, |map, field| {
            let bool_str = if value { "true" } else { "false" };
            map.serialize_entry(field, bool_str)
        });
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.serialize_field(field, |map, field| {
            map.serialize_entry(field, &crate::utils::SerializeDisplay(value))
        });
    }
}
