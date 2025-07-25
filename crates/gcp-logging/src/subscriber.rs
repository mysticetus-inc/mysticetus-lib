use std::sync::{Arc, OnceLock};

use tracing_core::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry, layer};

use crate::options::{DefaultLogOptions, LogOptions};
use crate::records::Records;

pub mod builder;
pub mod filter;
pub mod handle;
pub mod writer;

pub use filter::Filter;
pub use handle::{Handle, WeakHandle};
pub use writer::{MakeWriter, StdoutWriter};

mod event;
mod format_layer;
mod registry;
use format_layer::FormatLayer;

pub struct Subscriber<
    Opt: LogOptions = DefaultLogOptions,
    F = LevelFilter,
    MkWriter: MakeWriter = StdoutWriter,
> {
    // we can't dig into the internals of [layer::Layered],
    // so we need to hold a reference to the internal [Shared] instance.
    shared: Arc<Shared<Opt>>,
    // even with zero sized generics, this type easily approaches ~600b in size,
    // so put behind a box.
    registry: Box<layer::Layered<F, layer::Layered<FormatLayer<Opt, MkWriter>, Registry>>>,
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

impl<Opt, F, MkWriter> Subscriber<Opt, F, MkWriter>
where
    Opt: LogOptions,
    MkWriter: MakeWriter,
    F: Filter<Opt, MkWriter>,
{
    /// Creates a new [Handle] to this [Subscriber]
    pub fn handle(&self) -> Handle<Opt> {
        Handle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Creates a new [WeakHandle] to this [Subscriber]
    pub fn weak_handle(&self) -> WeakHandle<Opt> {
        WeakHandle {
            shared: Arc::downgrade(&self.shared),
        }
    }

    /// Installs 'self' as the global subscriber, panicking if one is already set.
    /// Returns a [Handle] to 'self'.
    ///
    /// Essentially just defers to [SubscriberInitExt::init].
    pub fn init(self) -> Handle<Opt> {
        let handle = self.handle();
        SubscriberInitExt::init(self);
        handle
    }

    /// Tries to install 'self' as the global subscriber, returning [Err] if
    /// there's already one set.
    ///
    /// Defers to [SubscriberInitExt::try_init], but returns a [Handle] on success.
    pub fn try_init(self) -> Result<Handle<Opt>, tracing_subscriber::util::TryInitError> {
        let handle = self.handle();
        SubscriberInitExt::try_init(self)?;
        Ok(handle)
    }

    /// Boils down to [tracing_core::dispatcher::with_default], but passes in a
    /// [Handle] to the subscriber to the scope fn.
    pub fn with_default<O>(self, scope: impl FnOnce(Handle<Opt>) -> O) -> O {
        let handle = self.handle();
        let _guard = self.set_default();
        scope(handle)
    }

    fn from_builder(builder: builder::LoggingBuilder<Opt, F, MkWriter>) -> Self {
        let crate::LoggingBuilder {
            stage,
            filter,
            project_id,
            options,
            make_writer,
        } = builder;

        let shared = Arc::new(Shared {
            stage,
            options,
            records: Records::with_capacity(16),
            project_id: match project_id {
                Some(proj_id) => OnceLock::from(proj_id),
                None => OnceLock::new(),
            },
        });

        let fmt = FormatLayer {
            make_writer,
            shared: Arc::clone(&shared),
        };

        let registry = Box::new(filter.with_subscriber(fmt.with_subscriber(Registry::default())));

        Self { registry, shared }
    }
}

#[derive(Debug)]
struct Shared<O: LogOptions = DefaultLogOptions> {
    options: O,
    records: Records,
    stage: crate::Stage,
    project_id: OnceLock<&'static str>,
}

impl<F, Opt, MkWriter> tracing::Subscriber for Subscriber<Opt, F, MkWriter>
where
    Opt: LogOptions + 'static,
    MkWriter: MakeWriter,
    F: Filter<Opt, MkWriter> + 'static,
{
    #[inline]
    fn exit(&self, span: &tracing::span::Id) {
        self.registry.exit(span);
    }

    #[inline]
    fn enter(&self, span: &tracing::span::Id) {
        self.registry.enter(span);
    }

    #[inline]
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        self.registry.enabled(metadata)
    }

    #[inline]
    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        self.registry.new_span(span)
    }

    #[inline]
    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        self.registry.record(span, values);
    }

    #[inline]
    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        self.registry.record_follows_from(span, follows);
    }

    #[inline]
    fn event(&self, event: &tracing::Event<'_>) {
        self.registry.event(event);
    }

    #[inline]
    fn on_register_dispatch(&self, subscriber: &tracing::Dispatch) {
        self.registry.on_register_dispatch(subscriber);
    }

    #[inline]
    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        self.registry.register_callsite(metadata)
    }

    #[inline]
    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        self.registry.max_level_hint()
    }

    #[inline]
    fn event_enabled(&self, event: &tracing::Event<'_>) -> bool {
        self.registry.event_enabled(event)
    }

    #[inline]
    fn clone_span(&self, id: &tracing::span::Id) -> tracing::span::Id {
        self.registry.clone_span(id)
    }

    #[inline]
    fn try_close(&self, id: tracing::span::Id) -> bool {
        self.registry.try_close(id)
    }

    #[inline]
    fn current_span(&self) -> tracing_core::span::Current {
        self.registry.current_span()
    }

    #[inline]
    unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
        if std::any::TypeId::of::<Handle<Opt>>() == id {
            // Handle<Opt> is a repr(transparent) wrapper around Arc<Shared<Opt>>, so
            // this is valid
            return Some(&self.shared as *const Arc<Shared<Opt>> as *const ());
        }

        if std::any::TypeId::of::<Opt>() == id {
            return Some(&self.shared.options as *const Opt as *const ());
        }

        unsafe { self.registry.downcast_raw(id) }
    }
}
