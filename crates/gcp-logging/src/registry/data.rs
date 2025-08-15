use std::cell::Cell;
use std::num::NonZeroU64;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use hashbrown::HashTable;
use http::HeaderValue;
use http_body::Body;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use sharded_slab::Clear;
use sharded_slab::pool::Ref;
use tracing::Metadata;
use tracing::field::Field;
use tracing::span::{Attributes, Id};

use crate::Severity;
use crate::http_request::{HttpRequest, TRACE_CTX_HEADER, TraceHeader};
use crate::json::JsonValue;
use crate::registry::Records;
use crate::utils::ErrorPassthrough;

pub const REQUEST_KEY: &str = "__request__";

#[derive(Debug)]
pub struct Data {
    pub(super) id: Id,
    pub(super) parent: Option<Id>,
    pub(super) metadata: &'static Metadata<'static>,
    pub(super) ref_count: AtomicUsize,
    pub(super) follows: AtomicU64,
    pub(super) closing: AtomicBool,
    pub(super) inner: RwLock<DataInner>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            id: Id::from_non_zero_u64(NonZeroU64::MAX),
            parent: None,
            metadata: Self::EMPTY_METADATA,
            ref_count: AtomicUsize::new(0),
            follows: AtomicU64::new(0),
            closing: AtomicBool::new(false),
            inner: RwLock::new(DataInner {
                alert: false,
                trace: None,
                severity: None,
                http_request: None,
                values: fxhash::FxHashMap::default(),
            }),
        }
    }
}

impl Clear for Data {
    fn clear(&mut self) {
        if let Some(parent) = self.parent.take() {
            tracing::dispatcher::get_default(|dispatch| dispatch.try_close(parent.clone()));
        }

        self.id = Id::from_non_zero_u64(NonZeroU64::MAX);
        self.metadata = Self::EMPTY_METADATA;
        *self.closing.get_mut() = false;

        let follows = std::mem::replace(self.follows.get_mut(), 0);

        if let Some(follows) = NonZeroU64::new(follows) {
            tracing::dispatcher::get_default(|dispatch| {
                dispatch.try_close(Id::from_non_zero_u64(follows))
            });
        }

        self.inner.get_mut().clear();
    }
}

pub struct ReadData<'a> {
    data: &'a Data,
    guard: RwLockReadGuard<'a, DataInner>,
}

impl<'a> ReadData<'a> {
    fn new(data: &'a Data) -> Self {
        Self {
            guard: data.inner.read(),
            data,
        }
    }

    pub fn visit_fields<E>(
        &self,
        mut f: impl FnMut(&'static str, &JsonValue) -> Result<(), E>,
    ) -> Result<(), E> {
        for (key, value) in self.guard.values.iter() {
            f(key, value)?;
        }

        Ok(())
    }

    pub fn id(&self) -> &tracing::Id {
        &self.data.id
    }

    pub fn trace(&self, records: &Records) -> Option<TraceHeader<'_>> {
        let project_id = records.project_id.get()?;
        self.guard
            .trace
            .as_ref()
            .map(|header| TraceHeader::new(project_id, header))
    }

    pub fn http_request(&self) -> Option<&HttpRequest> {
        self.guard.http_request.as_ref()
    }

    pub fn severity_opt(&self) -> Option<Severity> {
        self.guard.severity
    }

    pub fn severity(&self) -> Severity {
        self.guard.severity.unwrap_or_else(|| {
            Severity::from_tracing(self.data.metadata.level().clone(), self.guard.alert)
        })
    }
}

pub struct WriteData<'a> {
    pub(super) data: &'a Data,
    pub(super) guard: Option<RwLockWriteGuard<'a, DataInner>>,
}

impl<'a> WriteData<'a> {
    fn new(data: &'a Data) -> Self {
        Self { guard: None, data }
    }

    pub fn visitor<'r>(
        &mut self,
        records: &'r Records,
        metadata: &'static tracing::Metadata<'static>,
    ) -> super::visitor::Visitor<'r, &mut Self> {
        super::visitor::Visitor {
            records,
            metadata,
            inner: self,
        }
    }

    pub(super) fn inner(&mut self) -> &mut DataInner {
        self.guard.get_or_insert_with(|| self.data.inner.write())
    }
}

#[derive(Debug)]
pub(super) struct DataInner {
    pub(super) alert: bool,
    pub(super) severity: Option<Severity>,
    pub(super) http_request: Option<HttpRequest>,
    pub(super) trace: Option<HeaderValue>,
    pub(super) values: fxhash::FxHashMap<&'static str, crate::json::JsonValue>,
}

impl Clear for DataInner {
    fn clear(&mut self) {
        self.alert = false;
        self.severity = None;
        self.http_request = None;
        self.trace = None;
        self.values.clear();
    }
}

impl Data {
    pub fn write(&self) -> WriteData<'_> {
        WriteData::new(self)
    }

    pub fn read(&self) -> ReadData<'_> {
        ReadData::new(self)
    }

    pub fn metadata(&self) -> &'static tracing::Metadata<'static> {
        self.metadata
    }

    pub fn parent(&self) -> Option<&Id> {
        self.parent.as_ref()
    }

    pub fn follows(&self) -> Option<Id> {
        NonZeroU64::new(self.follows.load(Ordering::Relaxed)).map(Id::from_non_zero_u64)
    }

    pub fn record(&self, span_records: &tracing::span::Record<'_>, records: &Records) {
        if !span_records.is_empty() {
            span_records.record(&mut super::visitor::Visitor {
                inner: self.write(),
                metadata: self.metadata,
                records,
            });
        }
    }

    pub(super) fn reset(
        &mut self,
        id: &Id,
        attrs: &Attributes<'_>,
        parent: Option<Id>,
        records: &Records,
    ) {
        self.id = id.clone();
        self.metadata = attrs.metadata();
        self.parent = parent;
        *self.closing.get_mut() = false;
        assert_eq!(*self.ref_count.get_mut(), 0);
        *self.ref_count.get_mut() = 1;

        attrs.record(&mut super::visitor::Visitor {
            metadata: self.metadata,
            inner: self.inner.get_mut(),
            records,
        });
    }

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

pub struct DataRef<'a> {
    refer: Ref<'a, Data>,
    records: &'a Records,
}

impl<'a> DataRef<'a> {
    #[inline]
    pub fn new(refer: Ref<'a, Data>, records: &'a Records) -> Self {
        Self { refer, records }
    }

    pub fn id(&self) -> &Id {
        &self.refer.id
    }

    pub fn visit_all<O>(self, visitor: impl FnMut(DataRef<'a>) -> ControlFlow<O>) -> Option<O> {
        let mut ids_visited = HashTable::<Id>::with_capacity(super::spans::span_stack_len());
        self.visit_inner(visitor, &mut ids_visited)
    }

    fn visit_follows_inner<O>(
        &self,
        visitor: impl FnMut(DataRef<'a>) -> ControlFlow<O>,
        visited: &mut HashTable<Id>,
    ) -> Option<O> {
        use hashbrown::hash_table::Entry;

        if let Some(follows) = self.follows() {
            match visited.entry(follows.into_u64(), |id| *id == follows, |i| i.into_u64()) {
                Entry::Vacant(vac) => _ = vac.insert(follows.clone()),
                Entry::Occupied(_) => return None,
            }

            if let Some(follows_data) = self.records.get(&follows) {
                if let Some(ret) = follows_data.visit_follows_inner(visitor, visited) {
                    return Some(ret);
                }
            }
        }

        None
    }

    fn visit_inner<O>(
        self,
        mut visitor: impl FnMut(DataRef<'a>) -> ControlFlow<O>,
        visited: &mut HashTable<Id>,
    ) -> Option<O> {
        use hashbrown::hash_table::Entry;

        let Self { records, refer } = self;

        macro_rules! visit {
            ($id:expr; $data:expr) => {
                match visited.entry($id.into_u64(), |id| *id == $id, |i| i.into_u64()) {
                    Entry::Vacant(vacant) => {
                        vacant.insert($id);
                        match visitor($data) {
                            ControlFlow::Break(out) => return Some(out),
                            ControlFlow::Continue(()) => (),
                        }
                    }
                    Entry::Occupied(_) => (),
                }
            };
        }

        let mut current = Some(refer);

        while let Some(curr) = current.take() {
            if let Some(ref parent) = curr.parent {
                current = records.data.get(super::id_to_idx(parent));
            }

            let c = DataRef {
                refer: curr,
                records,
            };

            if let Some(ret) = c.visit_follows_inner(&mut visitor, visited) {
                return Some(ret);
            }

            let id = super::idx_to_id(c.refer.key());

            visit!(id; c);
        }

        None
    }

    pub fn parents(self) -> Spans<'a> {
        Spans {
            records: self.records,
            current: self
                .refer
                .parent
                .as_ref()
                .and_then(|id| self.records.data.get(super::id_to_idx(id))),
        }
    }
}

impl std::ops::Deref for DataRef<'_> {
    type Target = Data;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.refer.deref()
    }
}

impl std::fmt::Debug for DataRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // typed to ensure we pass in &Data instead of &Ref<'_, Data>
        let data: &Data = &*self.refer;
        f.debug_struct("DataRef").field("data", data).finish()
    }
}

pub(crate) struct Spans<'a> {
    records: &'a Records,
    current: Option<Ref<'a, Data>>,
}

impl<'a> Iterator for Spans<'a> {
    type Item = DataRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let refer = self.current.take()?;

        self.current = refer
            .parent
            .as_ref()
            .and_then(|id| self.records.data.get(super::id_to_idx(id)));

        Some(DataRef {
            refer,
            records: self.records,
        })
    }
}

#[derive(Clone)]
struct RequestData {
    http: HttpRequest,
    trace_header: Option<HeaderValue>,
}

pub(crate) struct NewRequest {
    inner: Cell<Option<RequestData>>,
}

impl std::fmt::Debug for NewRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewRequest").finish_non_exhaustive()
    }
}

impl NewRequest {
    pub(crate) fn new<B: Body>(req: &http::Request<B>) -> Self {
        Self {
            inner: Cell::new(Some(RequestData {
                trace_header: req.headers().get(TRACE_CTX_HEADER).cloned(),
                http: HttpRequest::from_request(req),
            })),
        }
    }
}

impl super::visitor::VisitorInner for DataInner {
    #[inline]
    fn visit_json(&mut self, field: &Field, json: JsonValue) {
        self.values.insert(field.name(), json);
    }

    #[inline]
    fn visit_bool(&mut self, field: &Field, b: bool) {
        if field.name() == "alert" && b {
            self.alert = true;
        } else {
            self.visit_json(field, b.into());
        }
    }

    #[inline]
    fn visit_error(
        &mut self,
        records: &Records,
        metadata: &tracing::Metadata<'_>,
        field: &Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        if field.name() == super::REQUEST_KEY {
            if let Some(req_data) = value
                .downcast_ref::<ErrorPassthrough<NewRequest>>()
                .and_then(|ErrorPassthrough(req)| req.inner.take())
            {
                self.http_request = Some(req_data.http);
                self.trace = req_data.trace_header;
                return;
            }
        }

        super::visitor::default_visit_error(self, records, metadata, field, value);
    }
}

impl super::visitor::VisitorInner for WriteData<'_> {
    #[inline]
    fn visit_json(&mut self, field: &Field, json: JsonValue) {
        self.inner().visit_json(field, json);
    }

    #[inline]
    fn visit_bool(&mut self, field: &Field, b: bool) {
        self.inner().visit_bool(field, b);
    }

    #[inline]
    fn visit_serialize<S>(&mut self, field: &Field, value: &S)
    where
        S: serde::Serialize + ?Sized,
    {
        self.inner().visit_serialize(field, value);
    }

    #[inline]
    fn visit_error(
        &mut self,
        records: &Records,
        metadata: &tracing::Metadata<'_>,
        field: &Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.inner().visit_error(records, metadata, field, value);
    }
}
