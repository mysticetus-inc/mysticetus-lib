use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AlertFound {
    Yes,
    #[default]
    No,
}

pub(crate) fn serialize_event_payload<M, O>(
    map: &mut M,
    event: &tracing::Event<'_>,
    options: &O,
    stage: crate::Stage,
) -> Result<AlertFound, M::Error>
where
    M: SerializeMap,
    O: crate::LogOptions,
{
    let mut visitor = Visitor {
        map,
        alert: AlertFound::No,
        options,
        metadata: event.metadata(),
        error: None,
    };

    event.record(&mut visitor);

    if let Some(error) = visitor.error {
        return Err(error);
    }

    if options.include_stage(stage, event.metadata()) {
        visitor.map.serialize_entry("stage", &stage)?;
    }

    Ok(visitor.alert)
}

struct Visitor<'a, M: SerializeMap, O = crate::DefaultLogOptions> {
    map: &'a mut M,
    alert: AlertFound,
    options: &'a O,
    metadata: &'a tracing::Metadata<'a>,
    error: Option<M::Error>,
}

macro_rules! impl_simple_record_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $fn_name(&mut self, field: &Field, value: $arg_ty) {
                if self.error.is_none() {
                    self.error = self.map.serialize_entry(field.name(), &value).err();
                }
            }
        )*
    };
}

impl<M: SerializeMap, O: crate::LogOptions> Visit for Visitor<'_, M, O> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if self.error.is_none() {
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
            self.alert = AlertFound::Yes;
        } else if self.error.is_none() {
            self.error = self.map.serialize_entry(field.name(), &value).err();
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if self.error.is_none() {
            self.error = self
                .map
                .serialize_entry(field.name(), &crate::utils::JsonFloat(value))
                .err();
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if self.error.is_none() {
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
