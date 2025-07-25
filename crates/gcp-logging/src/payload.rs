use std::sync::Arc;

use serde::Deserialize;
use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};
use tracing_subscriber::registry::LookupSpan;

pub mod label;

pub(crate) use label::LABEL_PREFIX;

use crate::Stage;
use crate::records::{Data, Records};

const ALERT: &str = "alert";

#[derive(Debug, Default)]
pub struct ScopeStorage {
    request_trace_index: Option<usize>,
    scopes: Vec<Arc<Data>>,
}

impl ScopeStorage {
    pub fn clear(&mut self) {
        self.request_trace_index = None;
        self.scopes.clear();
    }
}

/*
#[derive(Debug, Default)]
pub struct EventInfo {
    labels: u8,
    alert: bool,
}

pub(crate) fn serialize_event_payload<M, S>(
    map: &mut M,
    ctx: &tracing_subscriber::layer::Context<'_, S>,
    event: &tracing::Event<'_>,
    options: impl crate::LogOptions,
    records: &Records,
    stage: Stage,
) -> Result<EventInfo, M::Error>
where
    for<'b> S: LookupSpan<'b> + tracing::Subscriber,
    M: SerializeMap + ?Sized,
{
    let mut visitor = Visitor {
        map,
        event_info: EventInfo::default(),
        options,
        metadata: event.metadata(),
        error: None,
    };

    event.record(&mut visitor);

    if let Some(error) = visitor.error.take() {
        return Err(error);
    }

    let Some(current) = ctx.lookup_current() else {
        return Ok(visitor.event_info);
    };

    let read_records = records.read();

    let parent_spans_nested = visitor
        .options
        .nest_parent_span_fields(stage, event.metadata());

    for span_ref in current.scope() {
        if let Some(data) = read_records.get(span_ref.id()) {
            data.emit_body(
                visitor.map,
                &mut visitor.event_info.labels,
                &mut visitor.event_info.alert,
                parent_spans_nested,
            )?;
        }
    }

    Ok(visitor.event_info)
}

enum SerMapState {
    PendingMap,
    PendingKey,
    PendingValue,
}

impl SerMapState {
    fn next(&self) -> Self {
        match self {
            Self::PendingKey => Self::PendingValue,
            Self::PendingValue | Self::PendingMap => Self::PendingKey,
        }
    }
}

struct MapVisitor<'a, M: SerializeMap + ?Sized> {
    state: SerMapState,
    map: &'a mut M,
    error: &'a mut Option<M::Error>,
}

impl<'de, M: SerializeMap + ?Sized> serde::de::DeserializeSeed<'de> for &mut MapVisitor<'_, M> {
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(&mut *self)?;
        self.state = self.state.next();
        Ok(())
    }
}

macro_rules! visit_non_key_value {
    ($self:expr; $variant:ident -> $value:expr) => {{
        match $self.state {
            SerMapState::PendingMap => Ok(()),
            SerMapState::PendingKey => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::$variant($value),
                &$self,
            )),
            SerMapState::PendingValue => {
                if let Err(error) = $self.map.serialize_value(&$value) {
                    *$self.error = Some(error);
                }
                Ok(())
            }
        }
    }};
    ($self:expr; $variant:ident) => {{
        match $self.state {
            SerMapState::PendingMap => Ok(()),
            SerMapState::PendingKey => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::$variant,
                &$self,
            )),
            SerMapState::PendingValue => {
                if let Err(error) = $self.map.serialize_value(&()) {
                    *$self.error = Some(error);
                }
                Ok(())
            }
        }
    }};
}

macro_rules! visit_key_or_value {
    ($self:expr; $value:expr) => {{
        visit_key_or_value!($self; $value => $value)
    }};
    ($self:expr; $value:expr => $key_value:expr) => {{
        match $self.state {
            SerMapState::PendingMap => Ok(()),
            SerMapState::PendingKey => {
                if let Err(error) = $self.map.serialize_value(&$key_value) {
                    *$self.error = Some(error);
                }
                Ok(())
            }
            SerMapState::PendingValue => {
                if let Err(error) = $self.map.serialize_value(&$value) {
                    *$self.error = Some(error);
                }
                Ok(())
            }
        }
    }};
}

impl<'de, M: SerializeMap + ?Sized> serde::de::Visitor<'de> for &mut MapVisitor<'_, M> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.state {
            SerMapState::PendingMap => formatter.write_str("a json object"),
            SerMapState::PendingKey => formatter.write_str("a json map key"),
            SerMapState::PendingValue => formatter.write_str("a json map value"),
        }
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        serde::de::IgnoredAny::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;

        visit_non_key_value!(self; Seq)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_non_key_value!(self; Float -> v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => itoa::Buffer::new().format(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => itoa::Buffer::new().format(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => std::str::from_utf8(v).map_err(E::custom)?)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => if v { "true" } else { "false" })
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_non_key_value!(self; Unit)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => itoa::Buffer::new().format(v))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_key_or_value!(self; v => itoa::Buffer::new().format(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_non_key_value!(self; Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match self.state {
            // we're only in this state right as we start a map
            SerMapState::PendingMap => self.state = SerMapState::PendingKey,
            // ignore any nested maps
            _ => {
                let _ = serde::de::IgnoredAny::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                return Ok(());
            }
        }

        while map.next_key_seed(&mut *self)?.is_some() {
            match self.state {
                SerMapState::PendingValue => map.next_value_seed(&mut *self)?,
                _ => _ = map.next_value::<serde::de::IgnoredAny>()?,
            }

            if self.error.is_some() {
                // drain the map if we get an error
                while map
                    .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                    .is_some()
                {}

                break;
            }
        }

        Ok(())
    }
}
 */
