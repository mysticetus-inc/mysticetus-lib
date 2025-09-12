use std::sync::{Arc, OnceLock};

use tracing_core::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::util::SubscriberInitExt;

use crate::options::{DefaultLogOptions, LogOptions};
use crate::registry::Records;

pub mod builder;
mod filter;
pub mod handle;
pub mod writer;

pub use handle::Handle;
pub use writer::{MakeWriter, StdoutWriter};

mod event;

pub struct Subscriber<MkWriter: MakeWriter = StdoutWriter> {
    filter: LevelFilter,
    make_writer: MkWriter,
    records: &'static Records,
}

impl Default for Subscriber {
    fn default() -> Self {
        Self::from_builder(builder::LoggingBuilder::new())
    }
}

impl Subscriber {
    pub const fn builder() -> builder::LoggingBuilder {
        builder::LoggingBuilder::new()
    }
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

impl<MkWriter: MakeWriter> Subscriber<MkWriter> {
    /// Creates a new [Handle] to this [Subscriber]
    pub fn handle(&self) -> Handle {
        Handle {
            records: &self.records,
        }
    }

    /// Installs 'self' as the global subscriber, panicking if one is already set.
    /// Returns a [Handle] to 'self'.
    ///
    /// Essentially just defers to [SubscriberInitExt::init].
    pub fn init(self) -> Handle {
        let handle = self.handle();
        SubscriberInitExt::init(self);
        handle
    }

    /// Tries to install 'self' as the global subscriber, returning [Err] if
    /// there's already one set.
    ///
    /// Defers to [SubscriberInitExt::try_init], but returns a [Handle] on success.
    pub fn try_init(self) -> Result<Handle, tracing_subscriber::util::TryInitError> {
        let handle = self.handle();
        SubscriberInitExt::try_init(self)?;
        Ok(handle)
    }

    /// Boils down to [tracing_core::dispatcher::with_default], but passes in a
    /// [Handle] to the subscriber to the scope fn.
    pub fn with_default<O>(self, scope: impl FnOnce(Handle) -> O) -> O {
        let handle = self.handle();
        let _guard = self.set_default();
        scope(handle)
    }

    fn from_builder<Opt>(builder: builder::LoggingBuilder<Opt, MkWriter>) -> Self
    where
        Opt: LogOptions + 'static,
    {
        let crate::LoggingBuilder {
            stage,
            filter,
            project_id,
            options,
            make_writer,
        } = builder;

        let records = Records::new_with(options, Some(stage), project_id);

        Self {
            filter,
            records,
            make_writer,
        }
    }
}

impl<MkWriter: MakeWriter> tracing::Subscriber for Subscriber<MkWriter> {
    #[inline]
    fn exit(&self, span: &tracing::span::Id) {
        self.records.exit(span);
    }

    #[inline]
    fn enter(&self, span: &tracing::span::Id) {
        self.records.enter(span);
    }

    #[inline]
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        &self.filter >= metadata.level() && self.records.enabled(metadata)
    }

    #[inline]
    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        self.records.new_span(span)
    }

    #[inline]
    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        self.records.record(span, values);
    }

    #[inline]
    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        self.records.record_follows_from(span, follows);
    }

    #[inline]
    fn event(&self, event: &tracing::Event<'_>) {
        let emitter = event::EventEmitter::new(self.records, event);

        if let Err(error) = emitter.emit(&self.make_writer) {
            eprintln!(
                "[gcp-logging] failed to write event to {} - {error}",
                std::any::type_name::<<MkWriter as MakeWriter>::Writer<'_>>()
            );
        }
    }

    #[inline]
    fn on_register_dispatch(&self, subscriber: &tracing::Dispatch) {
        self.records.on_register_dispatch(subscriber);
    }

    #[inline]
    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        self.records.register_callsite(metadata)
    }

    #[inline]
    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(self.filter)
    }

    #[inline]
    fn event_enabled(&self, event: &tracing::Event<'_>) -> bool {
        &self.filter >= event.metadata().level() && self.records.event_enabled(event)
    }

    #[inline]
    fn clone_span(&self, id: &tracing::span::Id) -> tracing::span::Id {
        self.records.clone_span(id)
    }

    #[inline]
    fn try_close(&self, id: tracing::span::Id) -> bool {
        self.records.try_close(id)
    }

    #[inline]
    fn current_span(&self) -> tracing_core::span::Current {
        self.records.current_span()
    }

    #[inline]
    unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
        if std::any::TypeId::of::<MkWriter>() == id {
            return Some(&self.make_writer as *const MkWriter as *const ());
        }

        if std::any::TypeId::of::<Handle>() == id {
            // Handle is a repr(transparent) wrapper around &'static Records, so
            // this is valid
            return Some(self.records as *const Records as *const ());
        }

        if std::any::TypeId::of::<Self>() == id {
            return Some(self as *const Self as *const ());
        }

        unsafe { self.records.downcast_raw(id) }
    }
}
