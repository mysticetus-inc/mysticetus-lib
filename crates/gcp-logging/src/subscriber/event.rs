use std::io::Write;
use std::ops::ControlFlow;

use serde::ser::SerializeMap;
use timestamp::Timestamp;
use tracing::field::Field;
use tracing::span::Id;

use super::RecordError;
use crate::options::TryGetBacktrace;
use crate::registry::{DataRef, ReadOptions, Records};
use crate::subscriber::MakeWriter;
use crate::utils::HexBytes;
use crate::{Severity, Stage, keys};

const LABEL_PREFIX: &str = "label.";

pub struct EventEmitter<'a, 'event> {
    pub(super) records: &'static Records,
    pub(super) event: &'event tracing::Event<'event>,
    pub(super) event_data: Option<DataRef<'a>>,
    pub(super) request_span_id: Option<Id>,
    pub(super) has_emitted_http: bool,
    pub(super) has_emitted_trace: bool,
}

impl<'a, 'event> EventEmitter<'a, 'event> {
    pub(super) fn new(records: &'static Records, event: &'event tracing::Event<'event>) -> Self {
        Self {
            records,
            event,
            event_data: records.event_data(event),
            request_span_id: crate::middleware::current_request_span_id(),
            has_emitted_http: false,
            has_emitted_trace: false,
        }
    }

    pub fn emit<M>(self, mk_writer: &M) -> Result<(), RecordError>
    where
        M: MakeWriter + ?Sized,
    {
        if M::NEEDS_BUFFERING {
            crate::utils::with_buffer(|buffer| {
                buffer.clear();
                self.emit_inner::<M>(buffer)?;

                if buffer.is_empty() {
                    return Err(RecordError::Io(std::io::ErrorKind::WriteZero.into()));
                }

                let mut writer = mk_writer.make_writer();
                writer.write_all(&buffer)?;
                writer.flush()?;
                Ok(()) as Result<(), RecordError>
            })?;
        } else {
            let mut writer = mk_writer.make_writer();
            self.emit_inner::<M>(&mut writer)?;
        }
        Ok(())
    }

    fn emit_inner<M>(self, writer: &mut (impl Write + ?Sized)) -> Result<(), RecordError>
    where
        M: MakeWriter + ?Sized,
    {
        use serde::Serializer;

        {
            let mut json_ser = serde_json::Serializer::new(&mut *writer);

            let ser = path_aware_serde::Serializer::new(&mut json_ser);

            let mut map = ser.serialize_map(None)?;

            self.emit_to_map(&mut map)?;
            map.end()?;
        }

        if M::APPEND_NEWLINE {
            writer.write_all(b"\n")?;
        }

        writer.flush()?;

        Ok(())
    }

    fn emit_data_ref<M>(
        &mut self,
        map: &mut M,
        data_ref: DataRef<'_>,
        options: &ReadOptions<'_>,
    ) -> Result<bool, RecordError>
    where
        M: SerializeMap<Error = path_aware_serde::Error<serde_json::Error>> + ?Sized,
    {
        let mut parent_span_has_labels = false;

        let read_data = data_ref.read();

        if !self.has_emitted_trace {
            if let Some(trace) = read_data.trace(self.records) {
                map.serialize_entry(keys::TRACE_KEY, &trace)?;
                self.has_emitted_trace = true;
            }
        }

        if !self.has_emitted_http && options.include_http_info(self.event.metadata()) {
            if let Some(http) = read_data.http_request() {
                map.serialize_entry(keys::HTTP_REQUEST_KEY, &http)?;
                self.has_emitted_http = true;
            }
        }

        read_data.visit_fields(|key, value| {
            if key.starts_with(LABEL_PREFIX) {
                parent_span_has_labels |= self.request_span_id.as_ref() != Some(data_ref.id());
                Ok(())
            } else {
                map.serialize_entry(key, &value)
            }
        })?;

        Ok(parent_span_has_labels)
    }

    fn emit_to_map<M>(mut self, map: &mut M) -> Result<(), RecordError>
    where
        M: SerializeMap<Error = path_aware_serde::Error<serde_json::Error>> + ?Sized,
    {
        let mut options = None;

        let (ts, severity, event_has_labels) = self.emit_event(map, &mut options)?;

        let options = options.unwrap_or_else(|| self.records.options());

        if options.include_timestamp(self.event.metadata()) {
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

            let ts = ts.unwrap_or_else(Timestamp::now);
            map.serialize_entry(keys::TIMESTAMP_KEY, &ProtoTimestamp(ts))?;
        }

        let mut parent_span_has_labels = false;

        if let Some(event_data) = self.event_data.take() {
            serialize_span_id(event_data.id(), map)?;
            event_data.visit_all(
                |data_ref| match self.emit_data_ref(map, data_ref, &options) {
                    Ok(parent_has_labels) => {
                        parent_span_has_labels |= parent_has_labels;
                        ControlFlow::Continue(())
                    }
                    Err(error) => ControlFlow::Break(error),
                },
            );
        } else {
            if let Some(ref id) = self.request_span_id {
                serialize_span_id(id, map)?;
            }

            for data_ref in self.records.scope_iter(self.request_span_id.clone()) {
                self.emit_data_ref(map, data_ref, &options)?;
            }
        }

        map.serialize_entry(Severity::KEY, severity.as_upper_str())?;

        if severity.should_alert() {
            map.serialize_entry(keys::ALERT_ERROR_NAME, keys::ALERT_ERROR_VALUE)?;
        }

        let source = SourceLocation::new(self.event.metadata());
        map.serialize_entry(keys::SOURCE_LOCATION_KEY, &source)?;

        let stage_label = options.include_stage(self.event.metadata());

        match (stage_label, event_has_labels, parent_span_has_labels) {
            (false, false, false) => (),
            (true, false, false) => {
                #[derive(serde::Serialize)]
                struct StageLabels {
                    stage: Stage,
                }

                map.serialize_entry(
                    keys::LABELS_KEY,
                    &StageLabels {
                        stage: self.records.stage(),
                    },
                )?;
            }
            _ => (),
        }

        Ok(())
    }

    fn emit_event<M>(
        &self,
        map: &mut M,
        opts: &mut Option<ReadOptions<'_>>,
    ) -> Result<(Option<Timestamp>, Severity, bool), RecordError>
    where
        M: SerializeMap<Error = path_aware_serde::Error<serde_json::Error>> + ?Sized,
    {
        let mut event_visitor =
            EventVisitor::new(&mut *map, self.event.metadata(), self.records, opts);

        self.event.record(&mut event_visitor);

        if let Some(error) = event_visitor.error.take() {
            return Err(RecordError::Json(error));
        }

        let event_has_labels = event_visitor.labels_found > 0;

        let severity = event_visitor.severity.unwrap_or_else(|| {
            Severity::from_tracing(self.event.metadata().level().clone(), event_visitor.alert)
        });

        Ok((event_visitor.timestamp, severity, event_has_labels))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation<'a> {
    pub file: Option<&'a str>,
    pub line: u32,
    pub function: &'a str,
}

impl<'a> SourceLocation<'a> {
    pub(crate) fn new(meta: &'a tracing::Metadata<'a>) -> Self {
        Self {
            file: meta.file(),
            line: meta.line().unwrap_or_default(),
            function: meta.target(),
        }
    }
}

#[derive(Debug)]
struct EventVisitor<'a, 'opts_borrow, 'opts, M: SerializeMap + ?Sized> {
    records: &'opts Records,
    metadata: &'static tracing::Metadata<'static>,
    alert: bool,
    map: &'a mut M,
    error: Option<M::Error>,
    options: &'opts_borrow mut Option<ReadOptions<'opts>>,
    severity: Option<Severity>,
    timestamp: Option<Timestamp>,
    labels_found: u8,
}

impl<'a, 'opts_borrow, 'opts, M: SerializeMap + ?Sized> EventVisitor<'a, 'opts_borrow, 'opts, M> {
    const fn new(
        map: &'a mut M,
        metadata: &'static tracing::Metadata<'static>,
        records: &'opts Records,
        options: &'opts_borrow mut Option<ReadOptions<'opts>>,
    ) -> Self {
        Self {
            map,
            metadata,
            records,
            options,
            error: None,
            timestamp: None,
            severity: None,
            alert: false,
            labels_found: 0,
        }
    }

    fn add_label(&mut self) {
        self.labels_found = self.labels_found.saturating_add(1);
    }

    fn record_inner<S: serde::Serialize>(
        &mut self,
        field: &Field,
        get_value: impl FnOnce(&mut Self) -> S,
    ) {
        if self.error.is_some() {
            return;
        }

        if field.name().starts_with(LABEL_PREFIX) {
            self.add_label();
        } else {
            let value = get_value(self);
            self.error = self.map.serialize_entry(field.name(), &value).err();
        }
    }

    fn try_get_bt(&mut self, error: &(dyn std::error::Error + 'static)) -> TryGetBacktrace {
        let opts = self.options.get_or_insert_with(|| self.records.options());
        opts.try_get_backtrace(self.metadata, error)
    }
}

impl<M: SerializeMap + ?Sized> tracing::field::Visit for EventVisitor<'_, '_, '_, M> {
    #[inline]
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_inner(field, |_| crate::utils::SerializeDebug(value))
    }

    #[inline]
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_inner(field, |_| crate::utils::JsonFloat(value));
    }

    #[inline]
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_inner(field, |_| value);
    }

    #[inline]
    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_inner(field, |_| value);
    }

    #[inline]
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_inner(field, |_| value);
    }

    #[inline]
    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record_inner(field, |_| value);
    }

    #[inline]
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == Severity::KEY {
            if let Some(severity) = Severity::from_str(value) {
                self.severity = Some(match self.severity {
                    Some(existing) => existing.worst(severity),
                    None => severity,
                });

                return;
            }
        }

        if field.name() == "timestamp" {
            if let Ok(ts) = Timestamp::from_datetime_str(value) {
                self.timestamp = Some(ts);
                return;
            }
        }

        self.record_inner(field, |_| value);
    }

    #[inline]
    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        match std::str::from_utf8(value) {
            Ok(s) => self.record_str(field, s),
            Err(_) => self.record_debug(field, &HexBytes(value)),
        }
    }

    #[inline]
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "alert" && value {
            self.alert = true;
        } else {
            self.record_inner(field, |_| value);
        }
    }

    #[inline]
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_inner(field, |this| {
            let try_get_bt = this.try_get_bt(value);
            crate::utils::SerializeErrorReprs::new(value, try_get_bt)
        })
    }

    #[cfg(feature = "valuable")]
    #[inline]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        if field.name() == Severity::KEY {
            if let Some(severity) = Severity::from_value(&value) {
                self.severity = Some(match self.severity {
                    Some(existing) => existing.worst(severity),
                    None => severity,
                });

                return;
            }
        }

        if field.name() == "alert" && matches!(value, valuable::Value::Bool(true)) {
            self.alert = true;
            return;
        }

        self.record_inner(field, |_| valuable_serde::Serializable::new(value))
    }
}

pub(crate) fn serialize_span_id<M>(span_id: &tracing::Id, map: &mut M) -> Result<(), M::Error>
where
    M: SerializeMap + ?Sized,
{
    let span_id_bytes = span_id.into_u64().to_be_bytes();

    let mut dst = [0_u8; 2 * std::mem::size_of::<u64>()];

    hex::encode_to_slice(span_id_bytes, &mut dst).expect("dst has enough space to encode 8 bytes");

    let hex_str =
        std::str::from_utf8(&dst).expect("successful hex encoding should always be valid ascii");

    map.serialize_entry(crate::keys::SPAN_ID_KEY, hex_str)
}

/*
fn serialize_span_data<'s, M, O>(
    map: &mut M,
    records: &Records,
    read_data: ReadData<'_>,
    event_data: &mut EventData<'_>,
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
            if let Some((project_id, header)) =
                records.project_id.get().zip(trace.trace_header.as_ref())
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
                data: RefCell<&'a mut EventData<'s>>,
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

struct Visitor<'a, 's, M: SerializeMap + ?Sized> {
    records: &'static Records,
    map: &'a mut M,
    data: &'a mut EventData<'s>,
    metadata: &'a tracing::Metadata<'a>,
    error: Option<M::Error>,
}

impl<M: SerializeMap + ?Sized> Visitor<'_, '_, M> {
    fn record_inner(&mut self, field: &Field, value: impl serde::Serialize) {
        if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            self.error = self.map.serialize_entry(field.name(), &value).err();
        }
    }
}

impl<M: SerializeMap + ?Sized> Visit for Visitor<'_, '_, M> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_inner(field, crate::utils::SerializeDebug(value))
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "alert" && value {
            self.data.alert = true;
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_upper_str(value))
        {
            self.data.insert_span_severity(sev);
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_int(value as _))
        {
            self.data.insert_span_severity(sev);
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_int(value as _))
        {
            self.data.insert_span_severity(sev);
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_int(value as _))
        {
            self.data.insert_span_severity(sev);
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_int(value as _))
        {
            self.data.insert_span_severity(sev);
        } else {
            self.record_inner(field, value)
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_inner(field, crate::utils::JsonFloat(value))
    }

    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        match std::str::from_utf8(value) {
            Ok(s) => self.record_str(field, s),
            Err(_) => {
                let s = hex::encode(value);
                self.record_str(field, &s)
            }
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if field.name().starts_with(crate::payload::LABEL_PREFIX) {
            self.data.add_label();
        } else if self.error.is_none() {
            if field.name() == "alert" {
                self.data.alert = true;
            }

            let try_get_bt = self
                .data
                .options(self.records)
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

    #[cfg(feature = "valuable")]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        #[inline]
        const fn is_value_true(value: &valuable::Value<'_>) -> bool {
            match value {
                valuable::Value::Bool(true) => true,
                _ => false,
            }
        }

        if let Some(Some(sev)) =
            (field.name() == Severity::KEY).then(|| Severity::from_value(&value))
        {
            self.data.insert_span_severity(sev);
        } else if field.name() == "alert" && is_value_true(&value) {
            self.data.alert = true;
        } else {
            self.record_inner(field, valuable_serde::Serializable::new(value));
        }
    }
}
 */
