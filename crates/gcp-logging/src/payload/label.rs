use serde::Serializer;
use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};

use crate::{LogOptions, Stage};

pub(crate) const LABEL_PREFIX: &str = "label.";

#[derive(serde::Serialize)]
pub(crate) struct StageLabel {
    pub(crate) stage: Stage,
}

pub(crate) struct Labels<'a, O> {
    pub(crate) labels: u8,
    pub(crate) stage: Stage,
    pub(crate) event: &'a tracing::Event<'a>,
    pub(crate) options: O,
}

impl<O: LogOptions> serde::Serialize for Labels<'_, O> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let include_stage = self
            .options
            .include_stage(self.stage, self.event.metadata());

        let fields = self.labels as usize + include_stage as usize;

        let mut map = serializer.serialize_map(Some(fields))?;

        let mut visitor = LabelsVisitor {
            map: &mut map,
            error: None,
        };

        self.event.record(&mut visitor);

        if let Some(error) = visitor.error {
            return Err(error);
        }

        if include_stage {
            map.serialize_entry("stage", &self.stage)?;
        }

        map.end()
    }
}

struct LabelsVisitor<'a, M: SerializeMap> {
    map: &'a mut M,
    error: Option<M::Error>,
}

impl<M: SerializeMap> LabelsVisitor<'_, M> {
    #[inline]
    fn serialize_field(
        &mut self,
        field: &tracing::field::Field,
        serialize_entry: impl FnOnce(&mut M, &str) -> Result<(), M::Error>,
    ) {
        if self.error.is_some() {
            return;
        }

        if let Some(field) = field.name().strip_prefix(LABEL_PREFIX) {
            self.error = serialize_entry(self.map, field).err();
        }
    }

    #[cfg_attr(not(feature = "valuable"), allow(dead_code))]
    #[inline]
    fn serialize_field_value(
        &mut self,
        field: &tracing::field::Field,
        value: impl serde::Serialize,
    ) {
        if self.error.is_some() {
            return;
        }

        if let Some(field) = field.name().strip_prefix(LABEL_PREFIX) {
            self.error = self.map.serialize_entry(field, &value).err();
        }
    }
}

macro_rules! record_int_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name(&mut self, field: &Field, value: $arg_ty) {
                self.serialize_field(field, |map, field| {
                    let mut buf = itoa::Buffer::new();
                    map.serialize_entry(field, buf.format(value))
                });
            }
        )*
    };
}

impl<M: SerializeMap> Visit for LabelsVisitor<'_, M> {
    #[cfg(all(tracing_unstable, feature = "valuable"))]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        match value {
            // extract primitive types and direct to the relevant functions,
            // since a few of them need to be checked for special meaning
            // (i.e a field called 'alert' being a boolean true)
            valuable::Value::Bool(b) => self.record_bool(field, b),
            valuable::Value::Char(ch) => {
                let mut buf = [0; 4];
                let s = ch.encode_utf8(&mut buf);
                self.record_str(field, s)
            }
            valuable::Value::F32(float) => self.record_f64(field, float as f64),
            valuable::Value::F64(float) => self.record_f64(field, float),
            valuable::Value::I8(int) => self.record_i64(field, int as i64),
            valuable::Value::I16(int) => self.record_i64(field, int as i64),
            valuable::Value::I32(int) => self.record_i64(field, int as i64),
            valuable::Value::I64(int) => self.record_i64(field, int),
            valuable::Value::I128(int) => self.record_i128(field, int),
            valuable::Value::Isize(int) => self.record_i64(field, int as i64),
            valuable::Value::String(s) => self.record_str(field, s),
            valuable::Value::U8(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U16(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U32(uint) => self.record_u64(field, uint as u64),
            valuable::Value::U64(uint) => self.record_u64(field, uint),
            valuable::Value::U128(uint) => self.record_u128(field, uint),
            valuable::Value::Usize(uint) => self.record_u64(field, uint as u64),
            valuable::Value::Path(path) => match path.to_str() {
                Some(path_str) => self.record_str(field, path_str),
                None => self.serialize_field(field, |map, field| {
                    map.serialize_entry(field, &crate::utils::SerializeDisplay(&path.display()))
                }),
            },
            valuable::Value::Error(error) => self.record_error(field, error),
            valuable::Value::Unit => self.serialize_field_value(field, None::<()>),
            // wildcard includes all nested types, and any new types that get added since
            // valuable::Value is marked non exhaustive.
            value => self.serialize_field(field, |map, field| {
                map.serialize_entry(field, &valuable_serde::Serializable::new(value))
            }),
        }
    }

    #[inline]
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.serialize_field(field, |map, field| {
            map.serialize_entry(field, &crate::utils::SerializeDebug(value))
        });
    }

    record_int_fns! {
        record_i64(i64),
        record_i128(i128),
        record_u64(u64),
        record_u128(u128),
    }

    #[inline]
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.serialize_field(field, |map, field| {
            use std::num::FpCategory;

            let mut buf: ryu::Buffer;

            let float_str = match value.classify() {
                FpCategory::Normal | FpCategory::Subnormal => {
                    buf = ryu::Buffer::new();
                    buf.format_finite(value)
                }
                FpCategory::Zero => "0.0",
                FpCategory::Infinite if value.is_sign_positive() => "Inf",
                FpCategory::Infinite => "-Inf",
                FpCategory::Nan => "NaN",
            };

            map.serialize_entry(field, float_str)
        });
    }

    #[inline]
    fn record_str(&mut self, field: &Field, value: &str) {
        self.serialize_field(field, |map, field| map.serialize_entry(field, value));
    }

    #[inline]
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.serialize_field(field, |map, field| {
            let bool_str = if value { "true" } else { "false" };
            map.serialize_entry(field, bool_str)
        });
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.serialize_field(field, |map, field| {
            map.serialize_entry(field, &crate::utils::SerializeDisplay(value))
        });
    }
}
