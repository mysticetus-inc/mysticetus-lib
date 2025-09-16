//! Heavily based on [tracing_subscriber::Registry], but built for actually emitting logs
//! in GCP logging format.

use std::num::NonZeroU64;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

pub(crate) use data::{Data, DataRef, NewRequest, REQUEST_KEY};
use parking_lot::{RwLock, RwLockReadGuard};
use sharded_slab::pool::Pool;
use tracing::dispatcher::WeakDispatch;
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Event, Metadata};
use tracing_core::span::Current;

use crate::{DefaultLogOptions, LogOptions, Stage};

mod data;
mod iter;
mod spans;
mod visitor;

static RECORDS: OnceLock<Records> = OnceLock::new();
static DATA_POOL: OnceLock<Pool<Data>> = OnceLock::new();

pub(crate) struct Records {
    data: &'static Pool<Data>,
    pub(crate) stage: OnceLock<crate::Stage>,
    pub(crate) project_id: OnceLock<&'static str>,
    pub(crate) options: RwLock<Box<dyn LogOptions + 'static>>,
    dispatcher: RwLock<WeakDispatch>,
}

#[derive(Debug)]
pub struct ReadOptions<'a> {
    options: RwLockReadGuard<'a, Box<dyn LogOptions + 'static>>,
    stage: Stage,
}

impl ReadOptions<'_> {
    #[inline]
    pub fn include_stage(&self, meta: &Metadata<'_>) -> bool {
        self.options.include_stage(self.stage, meta)
    }

    #[inline]
    pub fn treat_as_error(&self, meta: &Metadata<'_>) -> bool {
        self.options.treat_as_error(meta)
    }

    #[inline]
    pub fn include_http_info(&self, meta: &Metadata<'_>) -> bool {
        self.options.include_http_info(meta)
    }

    #[inline]
    pub fn include_timestamp(&self, meta: &Metadata<'_>) -> bool {
        self.options.include_timestamp(self.stage, meta)
    }

    #[inline]
    pub fn parent_span_fields(&self, meta: &Metadata<'_>) -> crate::options::ParentSpanFields {
        self.options.parent_span_fields(self.stage, meta)
    }

    #[inline]
    pub fn try_get_backtrace(
        &self,
        meta: &Metadata<'_>,
        error: &(dyn std::error::Error + 'static),
    ) -> crate::options::TryGetBacktrace {
        self.options.try_get_backtrace(meta, error)
    }
}

impl Records {
    pub fn new() -> &'static Self {
        Self::new_with(DefaultLogOptions, None, None)
    }

    pub fn event_data(&self, event: &tracing::Event<'_>) -> Option<DataRef<'_>> {
        event
            .parent()
            .and_then(|id| self.get(id))
            .or_else(|| spans::current().and_then(|id| self.get(&id)))
    }

    pub fn scope_iter(&self, id: impl Into<Option<Id>>) -> iter::SpanDataIter<'_> {
        iter::SpanDataIter {
            records: self,
            next_id: id.into(),
        }
    }

    pub fn new_with(
        options: impl LogOptions + 'static,
        stage: Option<Stage>,
        project_id: Option<&'static str>,
    ) -> &'static Self {
        #[inline]
        fn opt_once_lock<T>(opt: Option<T>) -> OnceLock<T> {
            match opt {
                Some(value) => OnceLock::from(value),
                None => OnceLock::new(),
            }
        }

        let mut parts = Some((options, stage, project_id));

        let records = RECORDS.get_or_init(|| {
            let (options, stage, project_id) = parts.take().unwrap();

            Records {
                data: DATA_POOL.get_or_init(Pool::new),
                dispatcher: RwLock::new(tracing::Dispatch::none().downgrade()),
                stage: opt_once_lock(stage),
                project_id: opt_once_lock(project_id),
                options: RwLock::new(Box::new(options)),
            }
        });

        if let Some((opts, stage, project_id)) = parts {
            let boxed = Box::new(opts);
            *records.options.write() = boxed;

            if let Some(stage) = stage {
                _ = records.stage.set(stage);
            }

            if let Some(project_id) = project_id {
                _ = records.project_id.set(project_id);
            }
        }

        records
    }

    pub fn stage(&self) -> Stage {
        self.stage
            .get()
            .copied()
            .unwrap_or(const { Stage::detect() })
    }

    pub fn options(&self) -> ReadOptions<'_> {
        ReadOptions {
            options: self.options.read(),
            stage: self.stage(),
        }
    }

    pub(crate) fn fmt_inner(&self, mut dbg: std::fmt::DebugStruct<'_, '_>) -> std::fmt::Result {
        struct DebugScopes<'a> {
            next: &'a Id,
            spans: &'a [Id],
            records: &'a Records,
        }

        impl std::fmt::Debug for DebugScopes<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut dbg = f.debug_struct("Span");
                dbg.field("id", &self.next);

                match self.records.get(self.next) {
                    Some(data) => dbg.field("data", &data),
                    None => dbg.field("data", &"<closed>"),
                };

                // recurse another level
                if let Some((next, spans)) = self.spans.split_first() {
                    dbg.field(
                        "child",
                        &DebugScopes {
                            spans,
                            next,
                            records: self.records,
                        },
                    );
                }

                dbg.finish()
            }
        }

        {
            let guard = self.dispatcher.read();
            dbg.field("dispatcher", &*guard);
        }

        {
            let guard = self.options.read();
            dbg.field("options", &**guard);
        }

        if let Some(project_id) = self.project_id.get() {
            dbg.field("project_id", project_id);
        }

        spans::visit_slice(|spans| {
            if let Some((next, spans)) = spans.split_first() {
                dbg.field(
                    "scopes",
                    &DebugScopes {
                        next,
                        spans,
                        records: self,
                    },
                );
            }
        });

        dbg.finish()
    }
}

impl std::fmt::Debug for Records {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_inner(f.debug_struct("Records"))
    }
}

impl Records {
    pub(crate) fn dispatcher(&self) -> Option<tracing::Dispatch> {
        self.dispatcher.read().upgrade()
    }

    #[inline]
    pub(crate) fn get(&self, id: &Id) -> Option<DataRef<'_>> {
        let refer = self.data.get(id_to_idx(id))?;
        Some(DataRef::new(refer, self))
    }

    fn actually_close_span(&self, id: &Id, span: &DataRef<'_>) {
        if let Some(follows) = NonZeroU64::new(span.follows.swap(0, Ordering::SeqCst)) {
            self.with_dispatcher(|dispatcher| {
                dispatcher.try_close(Id::from_non_zero_u64(follows));
            });
        }

        self.data.clear(id_to_idx(id));
    }

    fn with_dispatcher<O>(&self, mut f: impl FnMut(&tracing::Dispatch) -> O) -> O {
        if let Some(dispatcher) = self.dispatcher() {
            f(&dispatcher)
        } else {
            tracing::dispatcher::get_default(f)
        }
    }
}

impl tracing::Subscriber for Records {
    #[inline]
    fn on_register_dispatch(&self, subscriber: &tracing::Dispatch) {
        *self.dispatcher.write() = subscriber.downgrade();
    }

    #[inline]
    fn event_enabled(&self, _: &Event<'_>) -> bool {
        true
    }

    #[inline]
    fn register_callsite(&self, _: &'static tracing::Metadata<'static>) -> Interest {
        Interest::always()
    }

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

        let mut data = self.data.create().expect("uh oh");
        let id = idx_to_id(data.key());

        #[cfg(feature = "debug-logging")]
        {
            println!("resetting data: {id:?}");
        }

        data.reset(&id, attrs, parent, self);
        drop(data);
        id
    }

    #[inline]
    fn event(&self, _: &Event<'_>) {
        debug_assert!(
            false,
            "<Records as Subscriber>::event should never be called"
        )
    }

    #[inline]
    fn record(&self, span: &Id, values: &Record<'_>) {
        if values.is_empty() {
            return;
        }

        if let Some(data) = self.get(span) {
            values.record(&mut visitor::Visitor {
                metadata: data.metadata,
                inner: data.write(),
                records: self,
            });
        } else {
            debug_assert!(
                false,
                "record called on non-existent span: {span:?} - {values:?}"
            );
        }
    }

    #[inline]
    fn record_follows_from(&self, span: &Id, follows: &Id) {
        if let Some(span) = self.get(span) {
            #[cfg(feature = "debug-logging")]
            {
                println!("record_follows_from {span:?} follows {follows:?}");
            }

            span.follows
                .store(self.clone_span(follows).into_u64(), Ordering::SeqCst);
        } else {
            debug_assert!(false, "couldn't find follows_from span: {span:?}");
        }
    }

    #[inline]
    fn current_span(&self) -> Current {
        spans::current()
            .and_then(|id| {
                let span = self.get(&id)?;
                Some(Current::new(id, span.metadata))
            })
            .unwrap_or_else(Current::none)
    }

    fn enter(&self, id: &Id) {
        #[cfg(feature = "debug-logging")]
        {
            println!("entering {id:?}");
        }
        if spans::enter(id) {
            self.clone_span(id);
        }
    }

    fn exit(&self, id: &Id) {
        #[cfg(feature = "debug-logging")]
        {
            println!("exiting {id:?}");
        }
        if spans::exit(id) {
            #[cfg(feature = "debug-logging")]
            {
                println!("exit - try_close {id:?}");
            }

            tracing::dispatcher::get_default(|dispatch| dispatch.try_close(id.clone()));
        }
    }

    fn clone_span(&self, id: &Id) -> Id {
        let Some(span) = self.get(id) else {
            panic!("can't clone span {id:?}, doesn't exist");
        };

        let prev_refs = span.ref_count.fetch_add(1, Ordering::Relaxed);

        #[cfg(feature = "debug-logging")]
        {
            println!(
                "clone_span - {id:?} ref_cnt - {}",
                prev_refs.wrapping_add(1)
            );
        }

        if prev_refs == 0 {
            panic!("closed span ({id:?}) shouldn't be cloned")
        }

        id.clone()
    }

    fn try_close(&self, id: Id) -> bool {
        let data = match self.get(&id) {
            Some(span) => span,
            None if std::thread::panicking() => return false,
            None => panic!("tried to drop a ref to {:?}, but no such span exists!", id),
        };

        // if we're actually closing, handle it separately
        if data
            .closing
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            self.actually_close_span(&id, &data);
            return true;
        }

        let refs = data.ref_count.fetch_sub(1, Ordering::Release);

        #[cfg(feature = "debug-logging")]
        {
            println!("try_close - {id:?} ref_cnt - {}", refs.wrapping_sub(1));
        }

        if !data.closing.load(Ordering::Relaxed) && !std::thread::panicking() {
            assert!(refs < usize::MAX, "reference count overflow!");
        }

        if refs > 1 {
            return false;
        }

        let closing = data
            .closing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();

        let follows_id =
            NonZeroU64::new(data.follows.swap(0, Ordering::SeqCst)).map(Id::from_non_zero_u64);

        std::sync::atomic::fence(Ordering::Acquire);

        if closing {
            self.with_dispatcher(|dispatch| {
                if let Some(ref follows) = follows_id {
                    #[cfg(feature = "debug-logging")]
                    {
                        println!("try_close dispatcher {id:?} - follows {follows_id:?}");
                    }

                    dispatch.try_close(follows.clone());
                }

                #[cfg(feature = "debug-logging")]
                {
                    println!("try_close dispatcher {id:?}");
                }
                dispatch.try_close(id.clone())
            })
        } else {
            false
        }
    }

    unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
        use std::any::TypeId;

        macro_rules! try_cast {
            ($t:ty => $getter:expr) => {{
                if id == TypeId::of::<$t>() {
                    let value: &$t = { $getter };
                    return Some(value as *const $t as *const ());
                }
            }};
        }

        try_cast!(Self => self);
        try_cast!(&'static Self => &Self::new());

        None
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
