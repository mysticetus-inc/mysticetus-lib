use std::cell::RefCell;
use std::sync::Arc;

use serde::ser::SerializeMap;
use timestamp::Timestamp;
use tracing::Event;
use tracing::field::{Field, Visit};
use tracing_subscriber::layer;
use tracing_subscriber::registry::{LookupSpan, SpanRef};

use super::Shared;
use crate::http_request::TraceHeader;
use crate::payload::LABEL_PREFIX;
use crate::payload::label::{Labels, StageLabel};
use crate::records::data::ReadData;
use crate::records::{Data, ReadRecords};
use crate::{DefaultLogOptions, LogOptions};

pub struct LogEvent<'shared, 'tracing, S: LookupSpan<'tracing>, O: LogOptions = DefaultLogOptions> {
    pub ctx: &'tracing layer::Context<'tracing, S>,
    pub event: &'tracing tracing::Event<'tracing>,
    data: EventData<'shared, O>,
}

#[derive(Debug)]
struct EventData<'shared, O: LogOptions> {
    shared: &'shared Shared<O>,
    alert: bool,
    emitted_http: bool,
    emitted_trace: bool,
    labels_found: u8,
}

impl<'s, O: LogOptions> EventData<'s, O> {
    const fn new(shared: &'s Shared<O>) -> Self {
        Self {
            shared,
            alert: false,
            labels_found: 0,
            emitted_http: false,
            emitted_trace: false,
        }
    }

    // defers to the inner Shared::options LogOptions method
    fn include_timestamp(&self, meta: &tracing::Metadata<'_>) -> bool {
        self.shared
            .options
            .include_timestamp(self.shared.stage, meta)
    }

    fn include_stage(&self, meta: &tracing::Metadata<'_>) -> bool {
        self.shared.options.include_stage(self.shared.stage, meta)
    }

    fn should_alert(&self, meta: &tracing::Metadata<'_>) -> bool {
        self.alert && self.shared.options.treat_as_error(meta)
    }

    fn add_label(&mut self) {
        self.labels_found = self.labels_found.saturating_add(1);
    }
}

impl<'shared, 'tracing, O: LogOptions, S> LogEvent<'shared, 'tracing, S, O>
where
    for<'s> S: LookupSpan<'s> + tracing::Subscriber,
{
    pub(super) fn new(
        shared: &'shared Shared<O>,
        ctx: &'tracing layer::Context<'tracing, S>,
        event: &'tracing Event<'tracing>,
    ) -> Self {
        Self {
            ctx,
            event,
            data: EventData::new(shared),
        }
    }

    pub(super) fn serialize<M>(mut self, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap + ?Sized,
    {
        self.serialize_preamble(map)?;

        let pre_span_label_count = self.data.labels_found;

        if let Some(span_ref) = self.ctx.event_span(self.event) {
            self.serialize_span_data(&span_ref, map)?;
            // if some of the parent spans contained labels, we need to
            // go back in and find + emit them.
            if self.data.labels_found != pre_span_label_count {
                self.serialize_event(map)?;
                return self.serialize_span_labels(&span_ref, map);
            }
        }

        self.serialize_event(map)?;
        self.serialize_event_labels(map)
    }

    fn read_scope_data(&mut self, span: &SpanRef<'_, S>, mut buf: buf::ScopeData<'_>) {
        let mut scope = span.scope();

        // ensure we have at least 1 parent before getting the read lock
        let Some(parent) = scope.next() else {
            return;
        };

        
        let read_records = self.data.shared.records.read();

        if let Some(data) = read_records.get(parent.id()) {
            
        }
    }

    fn serialize_span_data<M>(&mut self, span: &SpanRef<'_, S>, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap + ?Sized,
    {
        let mut scope = span.scope();

        // ensure we have at least 1 parent before getting the read lock
        let Some(parent) = scope.next() else {
            return Ok(());
        };

        let parent_span_fields = self
            .data
            .shared
            .options
            .parent_span_fields(self.data.shared.stage, span.metadata());

        let read_records = self.data.shared.records.read();

        // serialize the parent scope
        serialize_active_span_data(
            map,
            parent,
            &read_records,
            &mut self.data,
            parent_span_fields,
        )?;

        // then serialize the remaining
        for span in scope {
            let init_label_count = self.data.labels_found;

            if let Some(data) = serialize_active_span_data(
                map,
                span,
                &read_records,
                &mut self.data,
                parent_span_fields,
            )? {
                if init_label_count != self.data.labels_found || self.data.labels_found == u8::MAX {
                    todo!("cache span with labels");
                }
            }
        }

        Ok(())
    }

    fn serialize_preamble<M>(&mut self, map: &mut M) -> Result<(), M::Error>
    where
        M: serde::ser::SerializeMap + ?Sized,
    {
        let meta = self.event.metadata();

        if self.data.include_timestamp(meta) {
            let timestamp = ProtoTimestamp(Timestamp::now());
            map.serialize_entry(crate::types::TIMESTAMP_KEY, &timestamp)?;
        }

        map.serialize_entry(
            crate::types::SEVERITY_KEY,
            crate::types::get_severity_string(meta.level()),
        )?;

        map.serialize_entry(
            crate::types::SOURCE_LOCATION_KEY,
            &crate::types::SourceLocation::new(meta),
        )?;

        Ok(())
    }

    fn serialize_event<M>(&mut self, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap + ?Sized,
    {
        let mut event_visitor = Visitor {
            map,
            metadata: self.event.metadata(),
            data: &mut self.data,
            error: None,
        };

        self.event.record(&mut event_visitor);

        if let Some(error) = event_visitor.error {
            return Err(error);
        }

        // indicate this is an alert if we need to. We only listen to
        // 'alert = true' inside the actual event, not in any parent span data.
        if self.data.should_alert(self.event.metadata()) {
            map.serialize_entry(
                crate::types::ALERT_ERROR_NAME,
                crate::types::ALERT_ERROR_VALUE,
            )?;
        }

        Ok(())
    }

    fn serialize_event_labels<M>(&mut self, map: &mut M) -> Result<(), M::Error>
    where
        M: serde::ser::SerializeMap + ?Sized,
    {
        if self.data.labels_found > 0 {
            map.serialize_entry(
                crate::types::LABELS_KEY,
                &Labels {
                    labels: self.data.labels_found,
                    stage: self.data.shared.stage,
                    event: self.event,
                    options: self.data.shared.options,
                },
            )?;
        } else if self.data.include_stage(self.event.metadata()) {
            map.serialize_entry(
                crate::types::LABELS_KEY,
                &StageLabel {
                    stage: self.data.shared.stage,
                },
            )?;
        }

        Ok(())
    }

    fn serialize_span_labels<M>(
        &mut self,
        span: &SpanRef<'_, S>,
        map: &mut M,
    ) -> Result<(), M::Error>
    where
        M: SerializeMap + ?Sized,
    {
        todo!()
    }
}

fn serialize_span_data<'s, M, O>(
    map: &mut M,
    read_data: ReadData<'_>,
    event_data: &mut EventData<'_, O>,
    parent_span_fields: crate::options::ParentSpanFields,
) -> Result<(), M::Error>
where
    M: SerializeMap + ?Sized,
    O: LogOptions,
{
    if let Some(trace) = read_data.request_trace() {
        if !event_data.emitted_http {
            event_data.emitted_http = true;
            map.serialize_entry(crate::types::HTTP_REQUEST_KEY, &trace.request)?;
        }

        if !event_data.emitted_trace {
            if let Some((project_id, header)) = event_data
                .shared
                .project_id
                .get()
                .zip(trace.trace_header.as_ref())
            {
                event_data.emitted_trace = true;
                map.serialize_entry(
                    crate::types::TRACE_KEY,
                    &TraceHeader::new(project_id, header),
                )?;
            }
        }
    }

    match parent_span_fields {
        crate::options::ParentSpanFields::Nested => {
            struct SerializeNested<'a, 's, O: LogOptions> {
                data: RefCell<&'a mut EventData<'s, O>>,
                read_data: &'a ReadData<'a>,
            }

            impl<O: LogOptions> serde::Serialize for SerializeNested<'_, '_, O> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let mut map = serializer.serialize_map(None)?;

                    for (field, value) in self.read_data.data() {
                        if field.starts_with(LABEL_PREFIX) {
                            self.data.borrow_mut().add_label();
                            continue;
                        }

                        map.serialize_entry(field, value)?;
                    }

                    map.end()
                }
            }

            map.serialize_entry(
                read_data.span_name(),
                &SerializeNested {
                    data: RefCell::new(&mut *event_data),
                    read_data: &read_data,
                },
            )?;
        }
        crate::options::ParentSpanFields::Prefixed => {
            let span_name = read_data.span_name();

            for (field, value) in read_data.data() {
                if field.starts_with(LABEL_PREFIX) {
                    event_data.add_label();
                    continue;
                }

                map.serialize_entry(&format_args!("{span_name}.{field}"), value)?;
            }
        }
        crate::options::ParentSpanFields::Flattened => {
            for (field, value) in read_data.data() {
                if field.starts_with(LABEL_PREFIX) {
                    event_data.add_label();
                    continue;
                }

                map.serialize_entry(field, value)?;
            }
        }
    }
    Ok(())
}

fn serialize_active_span_data<'data, 's, M, S, O>(
    map: &mut M,
    span_ref: SpanRef<'s, S>,
    records: &'data ReadRecords<'_>,
    event_data: &mut EventData<'_, O>,
    parent_span_fields: crate::options::ParentSpanFields,
) -> Result<Option<&'data Arc<Data>>, M::Error>
where
    M: SerializeMap + ?Sized,
    S: LookupSpan<'s>,
    O: LogOptions,
{
    let Some(data) = records.get(span_ref.id()) else {
        return Ok(None);
    };

    let Some(read_data) = data.read() else {
        return Ok(None);
    };

    serialize_span_data(map, read_data, event_data, parent_span_fields)?;

    Ok(Some(data))
}

struct ProtoTimestamp(Timestamp);

impl serde::Serialize for ProtoTimestamp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Timestamp::serialize_as_proto(&self.0, serializer)
    }
}

struct Visitor<'a, 's, M: SerializeMap + ?Sized, O: LogOptions> {
    map: &'a mut M,
    data: &'a mut EventData<'s, O>,
    metadata: &'a tracing::Metadata<'a>,
    error: Option<M::Error>,
}

macro_rules! impl_simple_record_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $fn_name(&mut self, field: &Field, value: $arg_ty) {
                if field.name().starts_with(crate::payload::LABEL_PREFIX) {
                    self.data.add_label();
                } else if self.error.is_none() {
                    self.error = self.map.serialize_entry(field.name(), &value).err();
                }
            }
        )*
    };
}

impl<M: SerializeMap + ?Sized, O: LogOptions> Visit for Visitor<'_, '_, M, O> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            self.error = self
                .map
                .serialize_entry(field.name(), &crate::utils::SerializeDebug(value))
                .err();
        }
    }

    impl_simple_record_fns! {
        record_u64(u64),
        record_u128(u128),
        record_i64(i64),
        record_i128(i128),
        record_str(&str),
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "alert" && value {
            self.data.alert = true;
        } else if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            self.error = self.map.serialize_entry(field.name(), &value).err();
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            self.error = self
                .map
                .serialize_entry(field.name(), &crate::utils::JsonFloat(value))
                .err();
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            let try_get_bt = self
                .data
                .shared
                .options
                .try_get_backtrace(self.metadata, value);

            self.error = self
                .map
                .serialize_entry(
                    field.name(),
                    &crate::utils::SerializeErrorReprs::new(value, try_get_bt),
                )
                .err();
        }
    }
}

mod buf {
    use std::cell::RefCell;
    use std::sync::Arc;

    use crate::records::Data;

    pub(super) struct ScopeData<'a>(&'a mut Vec<Arc<Data>>);

    impl<'a> ScopeData<'a> {
        #[inline]
        pub(super) fn reborrow(&mut self) -> ScopeData<'_> {
            ScopeData(&mut *self.0)
        }
    }

    impl std::ops::Deref for ScopeData<'_> {
        type Target = Vec<Arc<Data>>;
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    impl std::ops::DerefMut for ScopeData<'_> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0
        }
    }

    // need to ensure the inner vec is cleared so we don't hold onto references
    // to span data that might have already been dropped. This serves as
    // an alternative to storing Weak<Data>, which is a bit more annoying
    // to deal with
    impl Drop for ScopeData<'_> {
        fn drop(&mut self) {
            self.0.clear();
        }
    }

    pub(super) fn with_buffer<O>(f: impl FnOnce(ScopeData<'_>) -> O) -> O {
        thread_local! {
            static BUF: RefCell<Vec<Arc<Data>>> = RefCell::new(Vec::with_capacity(8));
        }

        // Need to wrap the callback in an option so we don't have to move it into the
        // closure. If we did, there's no way to call with the fallback buffer it if
        // the TLS value is inaccessible for whatever reason.
        let mut callback = Some(f);

        let res = BUF.try_with(|buf| {
            if let Ok(mut ref_mut) = buf.try_borrow_mut() {
                let f = callback.take().expect("this is Some");
                ref_mut.clear();
                Some(f(ScopeData(&mut *ref_mut)))
            } else {
                None
            }
        });

        match res {
            Ok(Some(output)) => output,
            Err(_) | Ok(None) => {
                let mut tmp_buf = Vec::new();
                (callback.take().expect("this wasn't removed in the closure"))(ScopeData(
                    &mut tmp_buf,
                ))
            }
        }
    }
}
