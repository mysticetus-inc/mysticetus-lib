//! Heavily based on [tracing_subscriber::Registry], but built for actually emitting logs
//! in GCP logging format.

use std::cell::Cell;
use std::num::NonZeroU64;
use std::sync::atomic::AtomicUsize;

use parking_lot::RwLock;
use sharded_slab::Clear;
use sharded_slab::pool::{Pool, Ref};
use tracing::span::{Attributes, Id, Record, Span};
use tracing::{Event, Metadata};
use tracing_core::span::Current;
use tracing_subscriber::registry::{SpanData, SpanRef};

use crate::subscriber::{MakeWriter, StdoutWriter};
use crate::{DefaultLogOptions, LogOptions};

thread_local! {
    static ACTIVE_SPAN: Cell<Option<NonZeroU64>> = Cell::new(None);
}

pub struct Registry<Opts: LogOptions = DefaultLogOptions, MkWriter: MakeWriter = StdoutWriter> {
    data: Pool<Data>,
    options: Opts,
    mk_writer: MkWriter,
}

impl<O: LogOptions, MkWriter: MakeWriter> Registry<O, MkWriter> {
    #[inline]
    fn get(&self, id: &Id) -> Option<Ref<'_, Data>> {
        self.data.get(id_to_idx(id))
    }
}

impl Data {
    const EMPTY_METADATA: &'static Metadata<'static> = {
        struct FakeCallsite;
        const FAKE_CALLSITE: &'static FakeCallsite = &FakeCallsite;

        impl tracing::Callsite for FakeCallsite {
            fn metadata(&self) -> &Metadata<'_> {
                unreachable!()
            }

            fn set_interest(&self, _: tracing_core::Interest) {
                unreachable!()
            }
        }

        &Metadata::new(
            "",
            "",
            tracing::Level::TRACE,
            None,
            None,
            None,
            tracing::field::FieldSet::new(&[], tracing_core::identify_callsite!(FAKE_CALLSITE)),
            tracing::metadata::Kind::HINT,
        )
    };
}

#[derive(Debug)]
struct Data {
    parent: Option<Id>,
    metadata: &'static Metadata<'static>,
    ref_count: AtomicUsize,
    data: RwLock<fxhash::FxHashMap<&'static str, crate::json::JsonValue>>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            parent: None,
            metadata: Self::EMPTY_METADATA,
            ref_count: AtomicUsize::new(0),
            data: RwLock::new(fxhash::FxHashMap::default()),
        }
    }
}

impl Clear for Data {
    fn clear(&mut self) {
        if let Some(parent) = self.parent.take() {
            tracing::dispatcher::get_default(|dispatch| dispatch.try_close(parent.clone()));
        }

        self.metadata = Self::EMPTY_METADATA;

        self.data.get_mut().clear();
    }
}

impl<Opt: LogOptions, MkWriter: MakeWriter> tracing::Subscriber for Registry<Opt, MkWriter> {
    #[inline]
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let parent = if attrs.is_root() {
            None
        } else if attrs.is_contextual() {
            self.current_span().id().map(|id| self.clone_span(id))
        } else {
            attrs.parent().map(|id| self.clone_span(id))
        };

        let idx = self
            .data
            .create_with(|data| {
                data.metadata = attrs.metadata();
                data.parent = parent;
                assert_eq!(*data.ref_count.get_mut(), 0);
            })
            .expect("uh oh");

        idx_to_id(idx)
    }

    #[inline]
    fn event(&self, event: &Event<'_>) {}

    #[inline]
    fn record(&self, span: &Id, values: &Record<'_>) {}

    #[inline]
    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    #[inline]
    fn current_span(&self) -> Current {
        todo!()
    }

    #[inline]
    fn register_callsite(&self, _: &'static tracing::Metadata<'static>) -> tracing_core::Interest {
        tracing::subscriber::Interest::always()
    }

    fn enter(&self, id: &Id) {
        ACTIVE_SPAN.replace(Some(id.into_non_zero_u64()));
    }

    fn exit(&self, id: &Id) {
        let parent_span = self
            .get(id)
            .and_then(|data| data.parent.clone())
            .map(|id| id.into_non_zero_u64());

        ACTIVE_SPAN.set(parent_span);
    }
}

pub struct DataRef<'a>(Ref<'a, Data>);

fn get_fake_span_ref() -> &'static SpanRef<'static, tracing_subscriber::Registry> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tracing::Subscriber;
    use tracing_subscriber::registry::LookupSpan;

    static SPAN_REF_PTR: AtomicUsize = AtomicUsize::new(0);

    static HELPER_REGISTRY: std::sync::OnceLock<(tracing_subscriber::Registry, Id)> =
        std::sync::OnceLock::new();

    let (registry, id) = HELPER_REGISTRY.get_or_init(|| {
        let registry = tracing_subscriber::Registry::default();

        let value_set = Data::EMPTY_METADATA.fields().value_set(&[]);

        let attrs = tracing::span::Attributes::new(Data::EMPTY_METADATA, &value_set);

        let id = registry.new_span(&attrs);

        // enter the fake span, and never leave it
        registry.enter(&id);

        (registry, id)
    });

    let ptr = SPAN_REF_PTR.load(Ordering::Relaxed);

    // 0 is empty, 1 is initializing. >1 should be a pointer
    if ptr > 1 {
        return unsafe { &*(ptr as *const SpanRef<'static, tracing_subscriber::Registry>) };
    };

    if SPAN_REF_PTR
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let span_ref = Box::new(registry.span(id).expect("we know this exists"));

        let span_ref_ptr = Box::leak(span_ref);

        if SPAN_REF_PTR
            .compare_exchange(
                1,
                span_ref_ptr as *const _ as usize,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            panic!("failed to set the span")
        }

        return span_ref_ptr;
    }

    loop {
        let ptr = SPAN_REF_PTR.load(Ordering::SeqCst);
        if ptr > 1 {
            return unsafe { &*(ptr as *const SpanRef<'static, tracing_subscriber::Registry>) };
        }
        std::hint::spin_loop();
    }
}

impl<'a> SpanData<'a> for DataRef<'a> {
    #[inline]
    fn id(&self) -> Id {
        idx_to_id(self.0.key())
    }

    #[inline]
    fn parent(&self) -> Option<&Id> {
        self.0.parent.as_ref()
    }

    #[inline]
    fn metadata(&self) -> &'static Metadata<'static> {
        self.0.metadata
    }

    #[inline]
    fn is_enabled_for(&self, _: tracing_subscriber::filter::FilterId) -> bool {
        true
    }

    fn extensions(&self) -> tracing_subscriber::registry::Extensions<'_> {
        get_fake_span_ref().extensions()
    }

    fn extensions_mut(&self) -> tracing_subscriber::registry::ExtensionsMut<'_> {
        get_fake_span_ref().extensions_mut()
    }
}

#[inline]
const fn idx_to_id(idx: usize) -> Id {
    match idx.checked_add(1) {
        Some(id) => Id::from_non_zero_u64(
            NonZeroU64::new(id as u64).expect("just added 1, and didn't overflow"),
        ),
        None => panic!("overflowed usize::MAX spans"),
    }
}

#[inline]
const fn id_to_idx(id: &Id) -> usize {
    (id.into_non_zero_u64().get() - 1) as usize
}
