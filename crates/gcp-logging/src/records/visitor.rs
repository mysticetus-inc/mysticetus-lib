use std::error::Error as StdError;
use std::fmt::{self, Arguments};

use parking_lot::RwLockWriteGuard;
use tracing::field::{Field, Visit};

use super::data::{Data, WriteData};
use crate::LogOptions;
use crate::json::{JsonValue, Number, Primitive};

pub(crate) struct DataVisitor<'a, O> {
    pub(super) lock: Option<Option<WriteData<'a>>>,
    pub(super) data: &'a Data,
    pub(super) options: O,
    pub(super) metadata: &'a tracing::Metadata<'a>,
}

impl<'a, O: LogOptions> DataVisitor<'a, O> {
    // inner method for tracing::field::Visit::record_debug that gets around
    // funky temporaries related to format_args!
    fn record_debug_inner(&mut self, field: &Field, args: Arguments<'_>) {
        // if args can skip allocating, defer to Visit::record_str
        if let Some(s) = args.as_str() {
            self.record_str(field, s);
            return;
        }

        let mut buf = String::with_capacity(128);
        // this should never error, unless args formats to be isize::MAX bytes or more
        let _ = std::fmt::write(&mut buf, args);

        self.record(
            field,
            JsonValue::Primitive(Primitive::Str(buf.into_boxed_str())),
        )
    }

    fn record(&mut self, field: &Field, value: impl Into<JsonValue>) {
        fn record_inner<O: LogOptions>(
            visitor: &mut DataVisitor<'_, O>,
            field: &Field,
            value: JsonValue,
        ) {
            if let Some(guard) = visitor.lock.get_or_insert_with(|| visitor.data.write()) {
                guard.insert(field.name(), value);
            }
        }

        record_inner(self, field, value.into())
    }
}

impl<'a, O: LogOptions> Visit for DataVisitor<'a, O> {
    #[inline]
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record_debug_inner(field, format_args!("{value:?}"));
    }

    #[inline]
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record(field, value)
    }

    #[inline]
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record(field, Number::Float(value));
    }

    #[inline]
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record(field, Number::Int(value));
    }

    #[inline]
    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record(field, Number::BigInt(value));
    }

    #[inline]
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record(field, Number::Uint(value));
    }

    #[inline]
    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record(field, Number::BigUint(value));
    }

    #[inline]
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, Box::from(value));
    }

    #[inline]
    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        match std::str::from_utf8(value) {
            Ok(s) => self.record_str(field, s),
            Err(_) => self.record(field, hex::encode(value).into_boxed_str()),
        }
    }

    #[inline]
    fn record_error(&mut self, field: &Field, value: &(dyn StdError + 'static)) {
        let try_capture_bt = self.options.try_get_backtrace(self.metadata, value);

        let serialize_error = crate::utils::SerializeErrorReprs::new(value, try_capture_bt);

        match crate::json::JsonSerializer::try_serialize(serialize_error) {
            Some(json_repr) => self.record(field, json_repr),
            None => self.record_debug(field, value),
        }
    }

    #[cfg(all(tracing_unstable, feature = "valuable"))]
    #[inline]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        match crate::json::JsonSerializer::try_serialize(valuable_serde::Serializable::new(&value))
        {
            Some(json_repr) => self.record(field, json_repr),
            None => self.record_debug(field, &value),
        }
    }
}
