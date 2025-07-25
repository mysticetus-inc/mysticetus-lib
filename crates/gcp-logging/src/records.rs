use std::cell::Cell;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use http_body::Body;
use parking_lot::{RwLock, RwLockReadGuard};
use serde::ser::SerializeMap;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::registry::{LookupSpan, SpanRef};

use crate::LogOptions;
use crate::http_request::{RequestTrace, TraceHeader};
use crate::options::TryGetBacktrace;

pub(crate) mod data;
pub(crate) mod visitor;

pub(crate) use data::Data;

type RecordsInner = HashMap<SpanId, Arc<Data>, BuildHasherDefault<IdHasher>>;

pub(crate) struct Records {
    data: RwLock<RecordsInner>,
}

impl std::fmt::Debug for Records {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Records");

        match self.data.try_read() {
            Some(guard) => {
                let mut buf = itoa::Buffer::new();
                for (span, data) in guard.iter() {
                    dbg.field(buf.format(span.0.into_u64()), data);
                }

                dbg.finish()
            }
            None => dbg.finish_non_exhaustive(),
        }
    }
}

impl Records {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity_and_hasher(
                capacity,
                BuildHasherDefault::default(),
            )),
        }
    }

    pub fn read(&self) -> ReadRecords<'_> {
        ReadRecords {
            inner: self.data.read_recursive(),
        }
    }

    pub fn start_new_request(
        &self,
        id: tracing::span::Id,
        meta: &tracing::Metadata<'_>,
        parent_span: Option<tracing::Id>,
        trace: RequestTrace,
        options: impl LogOptions,
    ) -> ActiveRequest {
        ActiveRequest {
            start: Instant::now(),
            data: self.insert_new(
                id,
                meta,
                parent_span,
                None::<tracing::Event<'_>>,
                Some(trace),
                options,
            ),
        }
    }

    pub fn read_spans_into<D>(&self, spans: impl IntoIterator<Item = tracing::Id>, dst: &mut D)
    where
        D: Extend<Arc<Data>>,
    {
        let mut spans = spans.into_iter().peekable();

        // don't lock for an empty iterator
        if spans.peek().is_none() {
            return;
        };

        let read = self.read();

        let data_iter = spans
            .filter_map(|id| read.get(id))
            .filter(|data| !data.maybe_closed())
            .map(Arc::clone);

        dst.extend(data_iter);
    }

    pub fn insert_new<F: RecordFields>(
        &self,
        id: tracing::span::Id,
        meta: &tracing::Metadata<'_>,
        parent_span: Option<tracing::Id>,
        values: Option<F>,
        trace: Option<RequestTrace>,
        options: impl LogOptions,
    ) -> Arc<Data> {
        use std::collections::hash_map::Entry;

        let mut guard = self.data.write();

        match guard.entry(SpanId(id.clone())) {
            Entry::Vacant(vacant) => {
                let data = Arc::new(Data::new(id, meta, parent_span, trace));
                let ret = Arc::clone(vacant.insert(data));
                drop(guard);

                if let Some(values) = values {
                    values.record(&mut ret.visitor(meta, options));
                }

                ret
            }
            Entry::Occupied(occ) => {
                // clone ASAP, so we can drop the lock guard before inspecting the actual value.
                let existing = Arc::clone(occ.into_mut());
                drop(guard);
                existing.reset(id, meta, parent_span, values, trace, options);
                existing
            }
        }
    }
}

pub struct ReadRecords<'a> {
    inner: RwLockReadGuard<'a, RecordsInner>,
}

impl<'a> ReadRecords<'a> {
    pub fn get(&self, id: tracing::Id) -> Option<&Arc<Data>> {
        self.inner.get(&SpanId(id))
    }
}

#[derive(Default, Debug)]
struct IdHasher(u64);

impl std::hash::Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("SpanId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Newtype to ensure the Hash impl only calls write_u64
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct SpanId(tracing::span::Id);

impl std::hash::Hash for SpanId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.into_u64());
    }
}

#[derive(Debug)]
pub struct ActiveRequest {
    start: Instant,
    data: Arc<Data>,
}

impl ActiveRequest {
    pub fn update_from_response<B: Body>(self, resp: &http::Response<B>) {
        let elapsed = self.start.elapsed().into();
        let mut guard = self.data.inner.write();
        if let Some(ref mut trace) = guard.trace {
            trace.request.update_from_response(elapsed, resp);
        }
    }
}

impl Drop for ActiveRequest {
    fn drop(&mut self) {
        self.data.closed.store(true, Ordering::SeqCst);
    }
}
