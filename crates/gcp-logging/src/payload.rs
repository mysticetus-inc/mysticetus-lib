use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};

use super::types::LABEL_PREFIX;

const ALERT: &str = "alert";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct EventInfo {
    pub labels: u8,
    pub alert_found: bool,
}

pub(crate) fn serialize_event_payload<M, O>(
    map: &mut M,
    event: &tracing::Event<'_>,
    options: &O,
) -> Result<EventInfo, M::Error>
where
    M: SerializeMap,
    O: crate::LogOptions,
{
    let mut visitor = Visitor {
        map,
        event_info: EventInfo::default(),
        options,
        metadata: event.metadata(),
        error: None,
    };

    event.record(&mut visitor);

    if let Some(error) = visitor.error {
        return Err(error);
    }

    Ok(visitor.event_info)
}

struct Visitor<'a, M: SerializeMap, O = crate::DefaultLogOptions> {
    map: &'a mut M,
    event_info: EventInfo,
    options: &'a O,
    metadata: &'a tracing::Metadata<'a>,
    error: Option<M::Error>,
}

macro_rules! impl_simple_record_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $fn_name(&mut self, field: &Field, value: $arg_ty) {
                if field.name().starts_with(LABEL_PREFIX) {
                    self.event_info.labels = self.event_info.labels.saturating_add(1);
                } else if self.error.is_none() {
                    self.error = self.map.serialize_entry(field.name(), &value).err();
                }
            }
        )*
    };
}

impl<M: SerializeMap, O: crate::LogOptions> Visit for Visitor<'_, M, O> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name().starts_with(LABEL_PREFIX) {
            self.event_info.labels = self.event_info.labels.saturating_add(1);
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
        if field.name() == ALERT && value {
            self.event_info.alert_found = true;
        } else if field.name().starts_with(LABEL_PREFIX) {
            self.event_info.labels = self.event_info.labels.saturating_add(1);
        } else if self.error.is_none() {
            self.error = self.map.serialize_entry(field.name(), &value).err();
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name().starts_with(LABEL_PREFIX) {
            self.event_info.labels = self.event_info.labels.saturating_add(1);
        } else if self.error.is_none() {
            self.error = self
                .map
                .serialize_entry(field.name(), &crate::utils::JsonFloat(value))
                .err();
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if field.name().starts_with(LABEL_PREFIX) {
            self.event_info.labels = self.event_info.labels.saturating_add(1);
        } else if self.error.is_none() {
            let try_get_bt = self.options.try_get_backtrace(self.metadata, value);

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
