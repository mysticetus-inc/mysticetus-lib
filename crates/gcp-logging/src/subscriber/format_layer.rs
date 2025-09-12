use std::io::Write;
use std::sync::Arc;

use tracing_subscriber::field::RecordFields;
use tracing_subscriber::layer;
use tracing_subscriber::registry::LookupSpan;

use super::Shared;
use super::writer::MakeWriter;
use crate::options::LogOptions;
use crate::subscriber::event::LogEvent;

#[doc(hidden)]
pub struct FormatLayer<Opt: LogOptions, MkWriter: MakeWriter> {
    pub(super) make_writer: MkWriter,
    pub(super) shared: Arc<Shared<Opt>>,
}

/// Possible errors we could encounter when serializing and emitting a
/// log event to an arbitrary writer.
#[derive(Debug, thiserror::Error)]
enum RecordError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
}

impl<Opt, MkWriter> FormatLayer<Opt, MkWriter>
where
    Opt: crate::LogOptions + Copy,
    MkWriter: MakeWriter,
{
    fn serialize_entry<W, Sub>(
        &self,
        mut writer: W,
        event: &tracing::Event<'_>,
        ctx: layer::Context<'_, Sub>,
    ) -> Result<(), RecordError>
    where
        W: std::io::Write,
        for<'s> Sub: LookupSpan<'s> + tracing::Subscriber,
    {
        use serde::ser::{SerializeMap, Serializer};

        {
            let mut serializer = serde_json::Serializer::new(&mut writer);
            let serializer = path_aware_serde::Serializer::new(&mut serializer);

            let mut map = serializer.serialize_map(None)?;

            LogEvent::new(&self.shared, &ctx, event).serialize(&mut map)?;

            map.end()?;
        }

        writer.flush()?;

        Ok(())
    }

    fn try_record_event<S>(
        &self,
        event: &tracing::Event<'_>,
        ctx: layer::Context<'_, S>,
    ) -> Result<(), RecordError>
    where
        for<'s> S: LookupSpan<'s> + tracing::Subscriber,
    {
        if MkWriter::NEEDS_BUFFERING {
            crate::utils::with_buffer(|buffer| {
                buffer.clear();
                self.serialize_entry(&mut *buffer, event, ctx)?;

                let mut dst = self.make_writer.make_writer();
                dst.write_all(&buffer[..])?;
                Ok(())
            })
        } else {
            let writer = self.make_writer.make_writer();
            self.serialize_entry(writer, event, ctx)
        }
    }

    // common function used for both [layer::Layer::on_new_span] and
    // [layer::Layer::on_record]
    fn record<S>(&self, id: &tracing::Id, values: impl RecordFields, ctx: layer::Context<'_, S>)
    where
        S: tracing::Subscriber + std::fmt::Debug,
        for<'a> S: LookupSpan<'a, Data: std::fmt::Debug>,
    {
        // the span should always exist, and if not, its because we
        // shouldn't be in said span anymore.
        let Some(span_ref) = ctx.span(id) else {
            return;
        };

        self.shared.records.insert_new(
            id.clone(),
            span_ref.metadata(),
            span_ref.parent().map(|parent| parent.id()),
            Some(values),
            None,
            self.shared.options,
        );
    }
}

impl<S, Opt, MkWriter> layer::Layer<S> for FormatLayer<Opt, MkWriter>
where
    S: tracing::Subscriber + std::fmt::Debug,
    for<'a> S: LookupSpan<'a, Data: std::fmt::Debug>,
    Opt: crate::LogOptions + Copy,
    MkWriter: MakeWriter,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: layer::Context<'_, S>,
    ) {
        self.record(id, attrs, ctx);
    }

    fn on_record(
        &self,
        span: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: layer::Context<'_, S>,
    ) {
        self.record(span, values, ctx);
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: layer::Context<'_, S>) {
        if let Err(error) = self.try_record_event(event, ctx) {
            eprintln!("[gcp-logging] failed to record event: {error}");
        }
    }

    fn on_close(&self, id: tracing_core::span::Id, ctx: layer::Context<'_, S>) {}
}
