use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};

pub(crate) struct SerializePayload<'a, O = crate::DefaultLogOptions> {
    alert: std::cell::Cell<AlertFound>,
    metadata: &'a tracing::Metadata<'a>,
    options: &'a O,
    event: &'a tracing::Event<'a>,
    stage: crate::Stage,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AlertFound {
    Yes,
    #[default]
    No,
}

impl<'a, O> SerializePayload<'a, O> {
    pub fn new(
        metadata: &'a tracing::Metadata<'a>,
        event: &'a tracing::Event<'a>,
        options: &'a O,
        stage: crate::Stage,
    ) -> Self {
        Self {
            alert: std::cell::Cell::new(AlertFound::No),
            metadata,
            options,
            event,
            stage,
        }
    }

    pub fn alert(&self) -> AlertFound {
        self.alert.get()
    }
}

impl<O: crate::LogOptions> serde::Serialize for SerializePayload<'_, O> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        let mut visitor = Visitor {
            map: &mut map,
            alert: &self.alert,
            options: self.options,
            metadata: self.metadata,
            error: None,
        };

        self.event.record(&mut visitor);

        if let Some(error) = visitor.error {
            return Err(error);
        }

        if self.options.include_stage(self.stage, self.metadata) {
            map.serialize_entry("stage", &self.stage)?;
        }

        map.end()
    }
}

struct Visitor<'a, M: SerializeMap, O = crate::DefaultLogOptions> {
    map: &'a mut M,
    alert: &'a std::cell::Cell<AlertFound>,
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
            self.alert.set(AlertFound::Yes);
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
