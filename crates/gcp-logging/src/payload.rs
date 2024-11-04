use std::num::FpCategory;

use serde::ser::SerializeMap;
use tracing::field::{Field, Visit};

pub(crate) struct PayloadVisitor<'a, M: SerializeMap, O = crate::DefaultLogOptions> {
    alert: AlertFound,
    map_ser: &'a mut M,
    metadata: &'a tracing::Metadata<'a>,
    options: &'a O,
    error: Option<M::Error>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AlertFound {
    Yes,
    #[default]
    No,
}

impl<'a, M: SerializeMap, O: crate::LogOptions> PayloadVisitor<'a, M, O> {
    #[inline]
    pub(crate) fn new(
        metadata: &'a tracing::Metadata<'a>,
        map_ser: &'a mut M,
        options: &'a O,
    ) -> Self {
        Self {
            map_ser,
            alert: AlertFound::No,
            metadata,
            options,
            error: None,
        }
    }

    pub(crate) fn finish(self) -> Result<AlertFound, M::Error> {
        match self.error {
            None => Ok(self.alert),
            Some(error) => Err(error),
        }
    }

    #[inline]
    fn try_serialize<S: serde::Serialize>(&mut self, field: &str, value: S) {
        if let Err(error) = self.map_ser.serialize_entry(field, &value) {
            self.error = Some(error);
        }
    }
}

impl<M: SerializeMap, O: crate::LogOptions> Visit for PayloadVisitor<'_, M, O> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if self.error.is_none() {
            self.try_serialize(field.name(), crate::utils::SerializeDebug(value));
        }
    }

    fn record_error(&mut self, field: &Field, error: &(dyn std::error::Error + 'static)) {
        if self.error.is_none() {
            let try_get_bt = self.options.try_get_backtrace(self.metadata, error);

            self.try_serialize(
                field.name(),
                crate::utils::SerializeErrorReprs::new(error, try_get_bt),
            );
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if self.error.is_some() {
            return;
        }

        if field.name().eq_ignore_ascii_case("alert") && value {
            self.alert = AlertFound::Yes;
        } else {
            self.try_serialize(field.name(), value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if self.error.is_none() {
            self.try_serialize(field.name(), value);
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if self.error.is_some() {
            return;
        }

        match value.classify() {
            FpCategory::Zero | FpCategory::Subnormal | FpCategory::Normal => {
                self.try_serialize(field.name(), value)
            }
            FpCategory::Nan => self.try_serialize(field.name(), "NaN"),
            FpCategory::Infinite if value.is_sign_positive() => {
                self.try_serialize(field.name(), "Inf")
            }
            FpCategory::Infinite => self.try_serialize(field.name(), "-Inf"),
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if self.error.is_none() {
            self.try_serialize(field.name(), value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if self.error.is_none() {
            self.try_serialize(field.name(), value);
        }
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        if self.error.is_none() {
            self.try_serialize(field.name(), value);
        }
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        if self.error.is_none() {
            self.try_serialize(field.name(), value);
        }
    }
}
