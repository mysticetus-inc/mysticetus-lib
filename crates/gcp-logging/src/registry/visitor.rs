use tracing::field::{Field, Visit};

use crate::json::{JsonSerializer, JsonValue, Number, Primitive};
use crate::registry::Records;

pub(super) struct Visitor<'a, I> {
    pub(super) inner: I,
    pub(super) metadata: &'static tracing::Metadata<'static>,
    pub(super) records: &'a Records,
}

pub(super) trait VisitorInner {
    fn visit_json(&mut self, field: &Field, json: JsonValue);

    #[inline]
    fn visit_bool(&mut self, field: &Field, b: bool) {
        self.visit_json(field, b.into());
    }

    #[inline]
    fn visit_serialize<S>(&mut self, field: &Field, value: &S)
    where
        S: serde::Serialize + ?Sized,
    {
        let value = JsonSerializer::serialize(value);
        self.visit_json(field, value);
    }

    #[inline]
    fn visit_error(
        &mut self,
        records: &Records,
        metadata: &tracing::Metadata<'_>,
        field: &Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        default_visit_error(self, records, metadata, field, value);
    }

    #[cfg(feature = "valuable")]
    #[inline]
    fn visit_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        self.visit_serialize(field, &valuable_serde::Serializable::new(value));
    }
}

#[inline]
pub(super) fn default_visit_error<I: VisitorInner + ?Sized>(
    inner: &mut I,
    records: &Records,
    metadata: &tracing::Metadata<'_>,
    field: &Field,
    value: &(dyn std::error::Error + 'static),
) {
    let bt = records.options.read().try_get_backtrace(metadata, value);
    inner.visit_serialize(field, &crate::utils::SerializeErrorReprs::new(value, bt));
}

impl<V: VisitorInner + ?Sized> VisitorInner for &mut V {
    #[inline]
    fn visit_bool(&mut self, field: &Field, b: bool) {
        V::visit_bool(self, field, b);
    }

    #[inline]
    fn visit_json(&mut self, field: &Field, json: JsonValue) {
        V::visit_json(self, field, json);
    }

    #[inline]
    fn visit_serialize<S>(&mut self, field: &Field, value: &S)
    where
        S: serde::Serialize + ?Sized,
    {
        V::visit_serialize(self, field, value);
    }

    #[inline]
    fn visit_error(
        &mut self,
        records: &Records,
        metadata: &tracing::Metadata<'_>,
        field: &Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        V::visit_error(self, records, metadata, field, value);
    }

    #[cfg(feature = "valuable")]
    #[inline]
    fn visit_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        V::visit_value(self, field, value);
    }
}

impl<I: VisitorInner> Visit for Visitor<'_, I> {
    #[inline]
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        fn record_args<I>(inner: &mut I, field: &Field, value: std::fmt::Arguments<'_>)
        where
            I: VisitorInner + ?Sized,
        {
            inner.visit_serialize(field, &value);
        }

        record_args(&mut self.inner, field, format_args!("{:?}", value));
    }

    #[inline]
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.inner.visit_json(field, Number::Float(value).into());
    }

    #[inline]
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.inner.visit_json(field, Number::Int(value).into());
    }

    #[inline]
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.inner.visit_json(field, Number::Uint(value).into());
    }

    #[inline]
    fn record_i128(&mut self, field: &Field, value: i128) {
        self.inner.visit_json(field, Number::BigInt(value).into());
    }

    #[inline]
    fn record_u128(&mut self, field: &Field, value: u128) {
        self.inner.visit_json(field, Number::BigUint(value).into());
    }

    #[inline]
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.inner.visit_bool(field, value);
    }

    #[inline]
    fn record_str(&mut self, field: &Field, value: &str) {
        self.inner.visit_serialize(field, value);
    }

    #[inline]
    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        match std::str::from_utf8(value) {
            Ok(s) => self.record_str(field, s),
            Err(_) => {
                let hex = hex::encode(value);
                self.inner
                    .visit_json(field, Primitive::Str(hex.into_boxed_str()).into());
            }
        }
    }

    #[inline]
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.inner
            .visit_error(self.records, self.metadata, field, value);
    }

    #[cfg(feature = "valuable")]
    #[inline]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        self.inner.visit_value(field, value);
    }
}
