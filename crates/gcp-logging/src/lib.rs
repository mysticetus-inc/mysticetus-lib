use std::fmt;
use std::sync::Arc;

use dashmap::DashMap;
use serde::ser::SerializeMap;
use subscriber::SubscriberHandle;
use trace_layer::ActiveTraces;
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, SubscriberBuilder};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

pub mod env_filter;
// mod json_serializer;
mod live_ids;
mod payload;
mod registry;
pub mod response_error;
mod subscriber;
pub mod trace_layer;
//mod json;
mod http_request;
mod middleware;
mod middlewarev2;
mod span;
mod types;
mod utils;

#[inline]
pub fn init_logging(project_id: &'static str, stage: Stage) -> middlewarev2::TraceLayer {
    init_logging_opt(project_id, stage, DefaultLogOptions)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    #[default]
    Dev,
    Test,
    Production,
}

pub fn init_logging_opt<Options>(
    project_id: &'static str,
    stage: Stage,
    options: Options,
) -> middlewarev2::TraceLayer
where
    Options: LogOptions,
{
    let inner = Arc::new(DashMap::with_capacity_and_hasher(
        128,
        fxhash::FxBuildHasher::default(),
    ));

    let handle = SubscriberHandle::new(Arc::clone(&inner));

    let traces = ActiveTraces::new(inner);

    let formatter = GoogleLogEventFormatter {
        stage,
        project_id,
        traces: traces.clone(),
        options,
    };

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .json()
        .event_format(formatter)
        .finish()
        .with(traces)
        .init();

    middlewarev2::TraceLayer::new(handle)
}

pub fn init_with<F>(f: F) -> middleware::TraceLayer
where
    F: FnOnce(SubscriberBuilder),
{
    f(SubscriberBuilder::default());

    middleware::new_trace_layer()
}

#[derive(Debug, Clone)]
pub struct GoogleLogEventFormatter<O = DefaultLogOptions> {
    project_id: &'static str,
    stage: Stage,
    traces: ActiveTraces,
    options: O,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TryGetBacktrace {
    #[default]
    No,
    Yes,
    Force,
}

pub trait LogOptions: Send + Sync + Clone + 'static {
    fn include_http_info(&self, meta: &tracing::Metadata<'_>) -> bool;

    fn treat_as_error(&self, meta: &tracing::Metadata<'_>) -> bool;

    fn include_stage(&self, stage: Stage, meta: &tracing::Metadata<'_>) -> bool;

    fn try_get_backtrace(
        &self,
        meta: &tracing::Metadata<'_>,
        error: &(dyn std::error::Error + 'static),
    ) -> TryGetBacktrace;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultLogOptions;

impl LogOptions for DefaultLogOptions {
    fn include_http_info(&self, meta: &tracing::Metadata<'_>) -> bool {
        matches!(*meta.level(), tracing::Level::ERROR | tracing::Level::WARN)
    }

    fn treat_as_error(&self, meta: &tracing::Metadata<'_>) -> bool {
        matches!(*meta.level(), tracing::Level::ERROR)
    }

    fn include_stage(&self, _stage: Stage, _meta: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn try_get_backtrace(
        &self,
        meta: &tracing::Metadata<'_>,
        _error: &(dyn std::error::Error + 'static),
    ) -> TryGetBacktrace {
        if self.treat_as_error(meta) {
            TryGetBacktrace::Yes
        } else {
            TryGetBacktrace::No
        }
    }
}

struct JsonVisitor<'a, S: SerializeMap> {
    map: &'a mut S,
    error: Option<S::Error>,
}

impl<S: SerializeMap> JsonVisitor<'_, S> {
    fn try_serialize<V: serde::Serialize>(&mut self, name: &str, value: V) {
        if self.error.is_some() {
            return;
        }

        if let Err(err) = self.map.serialize_entry(name, &value) {
            self.error = Some(err);
        }
    }
}

impl<'a, S> Visit for JsonVisitor<'a, S>
where
    S: SerializeMap,
{
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.try_serialize(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.try_serialize(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.try_serialize(field.name(), value);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.try_serialize(field.name(), value);
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.try_serialize(field.name(), value);
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.try_serialize(
            field.name(),
            utils::SerializeErrorReprs::new(value, TryGetBacktrace::No),
        );
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.try_serialize(field.name(), value);
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        use std::num::FpCategory::*;
        // NaN and +/- Inf are invalid in Json, so we need to handle turning them into their string
        // reprs
        match value.classify() {
            Zero | Subnormal | Normal => self.try_serialize(field.name(), value),
            Nan => self.try_serialize(field.name(), "NaN"),
            Infinite if value.is_sign_negative() => self.try_serialize(field.name(), "Inf"),
            Infinite => self.try_serialize(field.name(), "-Inf"),
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.try_serialize(field.name(), utils::SerializeDebug(value));
    }
}

impl<Opts> GoogleLogEventFormatter<Opts>
where
    Opts: LogOptions,
{
    fn try_format_event_json<S, N>(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> serde_json::Result<()>
    where
        S: Subscriber,
        for<'a> S: LookupSpan<'a>,
        for<'b> N: FormatFields<'b> + 'static,
    {
        use serde::Serialize;

        let entry = types::LogEntry::new(
            self.project_id,
            ctx,
            self.stage,
            &self.options,
            &self.traces,
            event,
        );

        let mut serializer = serde_json::Serializer::new(utils::IoAdapter(writer));

        entry.serialize(&mut serializer)?;

        Ok(())
    }
}

impl<Opts, S, N> FormatEvent<S, N> for GoogleLogEventFormatter<Opts>
where
    Opts: LogOptions,
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
    for<'b> N: FormatFields<'b> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        if let Err(err) = self.try_format_event_json(ctx, &mut writer, event) {
            write!(writer, "error serializing log json: {err:#?}")?;
            if err.is_io() {
                return Err(fmt::Error);
            }
        }
        // acts like a flush
        writer.write_str("\n")
    }
}
