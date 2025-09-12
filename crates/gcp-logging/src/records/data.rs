use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing_subscriber::field::RecordFields;

use crate::LogOptions;
use crate::http_request::{RequestTrace, TraceHeader};
use crate::json::JsonValue;

#[derive(Debug)]
pub struct Data {
    closed: AtomicBool,
    inner: RwLock<DataInner>,
}

#[derive(Debug)]
pub(super) struct DataInner {
    pub(super) id: tracing::Id,
    pub(super) parent_span: Option<tracing::Id>,
    pub(super) span_name: &'static str,
    pub(super) trace: Option<RequestTrace>,
    pub(super) data: fxhash::FxHashMap<Field, JsonValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Field {
    pub(super) span_name: &'static str,
    pub(super) field_name: &'static str,
}

impl std::fmt::Display for Field {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.span_name)?;
        std::fmt::Write::write_char(f, '.')?;
        f.write_str(self.field_name)
    }
}

impl Data {
    pub(super) fn new(
        id: tracing::Id,
        meta: &tracing::Metadata<'_>,
        parent_span: Option<tracing::Id>,
        trace: Option<RequestTrace>,
    ) -> Self {
        Self {
            closed: AtomicBool::new(false),
            inner: RwLock::new(DataInner {
                id,
                span_name: meta.name(),
                trace,
                parent_span,
                data: fxhash::FxHashMap::default(),
            }),
        }
    }

    pub(crate) fn visitor<'a, O: LogOptions>(
        &'a self,
        metadata: &'a tracing::Metadata<'a>,
        options: O,
    ) -> super::visitor::DataVisitor<'a, O> {
        super::visitor::DataVisitor {
            lock: None,
            data: self,
            metadata,
            options,
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub(crate) fn id(&self) -> &tracing::Id {
        &self.id
    }

    pub(crate) fn write(&self) -> Option<WriteData<'_>> {
        if self.is_closed() {
            return None;
        }

        // then, get the actual lock and see if the inner generation matches
        let write = self.inner.write();

        if self.is_closed() {
            return None;
        }

        Some(WriteData { write })
    }

    pub(crate) fn read(&self) -> Option<ReadData<'_>> {
        if self.is_closed() {
            return None;
        }
        // then, get the actual lock and see if the inner generation matches
        let read = self.inner.read_recursive();

        if self.is_closed() {
            return None;
        }

        Some(ReadData { read })
    }

    pub(super) fn reset<F: RecordFields>(
        &self,
        id: tracing::span::Id,
        metadata: &tracing::Metadata<'_>,
        parent_span: Option<tracing::Id>,
        values: Option<F>,
        trace: Option<RequestTrace>,
        options: impl crate::LogOptions,
    ) {
        if self
            .closed
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            // todo!("add warning")
        }

        let mut guard = self.inner.write();
        guard.data.clear();

        guard.id = id;
        guard.parent_span = parent_span;
        guard.span_name = metadata.name();
        guard.trace = trace;

        if let Some(values) = values {
            values.record(&mut super::visitor::DataVisitor {
                lock: Some(Some(WriteData { write: guard })),
                data: self,
                metadata,
                options,
            });
        }
    }
}

pub(crate) struct WriteData<'a> {
    write: RwLockWriteGuard<'a, DataInner>,
}
impl<'a> WriteData<'a> {
    pub fn span_name(&self) -> &'static str {
        self.write.span_name
    }

    pub fn request_trace_mut(&mut self) -> Option<&mut RequestTrace> {
        self.write.trace.as_mut()
    }

    pub fn insert(&mut self, field_name: &'static str, value: JsonValue) {
        let span_name = self.write.span_name;
        self.write.data.insert(
            Field {
                span_name,
                field_name,
            },
            value,
        );
    }
}

pub(crate) struct ReadData<'a> {
    read: RwLockReadGuard<'a, DataInner>,
}

impl<'a> ReadData<'a> {
    pub(crate) fn request_trace(&self) -> Option<&RequestTrace> {
        self.read.trace.as_ref()
    }

    pub(crate) fn span_name(&self) -> &'static str {
        self.read.span_name
    }

    pub(crate) fn trace_header(&self, project_id: Option<&'static str>) -> Option<TraceHeader<'_>> {
        let header = self
            .read
            .trace
            .as_ref()
            .and_then(|trace| trace.trace_header.as_ref())?;

        let project_id = project_id?;

        Some(TraceHeader::new(project_id, header))
    }

    pub(crate) fn labels(&self) -> impl Iterator<Item = (&'static str, &'_ JsonValue)> + '_ {
        self.read.data.iter().filter_map(|(field, value)| {
            let label_name = field
                .field_name
                .strip_prefix(crate::payload::LABEL_PREFIX)?;

            Some((label_name, value))
        })
    }

    pub(crate) fn data(&self) -> DataIter<'_> {
        self.read.data.iter().map(map_field_name)
    }
}

#[inline]
const fn map_field_name<'b>(pair: (&Field, &'b JsonValue)) -> (&'static str, &'b JsonValue) {
    (pair.0.field_name, pair.1)
}

pub(crate) type DataIter<'a> = std::iter::Map<
    std::collections::hash_map::Iter<'a, Field, JsonValue>,
    for<'b> fn((&'b Field, &'b JsonValue)) -> (&'static str, &'b JsonValue),
>;
