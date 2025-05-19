use std::borrow::Cow;

use bytes::{BufMut, Bytes};
use serde::Serialize;

pub mod range;

pub use range::{Range, RangeKind};

pub enum Unsupported {}

pub enum Value<'a, T: Serialize + ?Sized + 'a = ()> {
    Bool(bool),
    Bytes(Bytes),
    Date(timestamp::Date),
    Time(timestamp::Time),
    DateTime(Unsupported),
    Timestamp(timestamp::Timestamp),
    Geography(),
    Integer(i64),
    Json(&'a T),
    Numeric(Unsupported),
    BigNumeric(Unsupported),
    String(Cow<'a, str>),
    Interval(timestamp::Duration),
    Range(RangeKind),
    Repeated(Cow<'a, [Value<'a, T>]>),
    Record(Unsupported),
}

impl<'a, T: Serialize + ?Sized + 'a> Value<'a, T> {
    pub fn encode<B: BufMut + ?Sized>(&self, buf: &mut B) -> serde_json::Result<()> {
        match self {
            Self::Bytes(bytes) => bytes.encode_raw(buf),
            Self::Bool(b) => b.encode_raw(buf),
            Self::Date(date) => date.days_since(timestamp::Date::UNIX_EPOCH).encode_raw(buf),
            Self::Timestamp(ts) => protos::protobuf::Timestamp::from(*ts).encode_raw(buf),
            Self::Integer(int) => int.encode_raw(buf),
            Self::Range(range) => range.encode(buf),
            Self::Interval(interval) => protos::protobuf::Duration::from(*interval).encode_raw(buf),
            Self::Time(time) => time.to_string().encode_raw(buf),
            Self::String(str) => str.encode_raw(buf),
            Self::Repeated(repeated) => repeated.encode_raw(buf),
            Self::Json(json) => serde_json::to_string(json)?.encode_raw(buf),
            Self::Geography() => todo!(),
            Self::Record(unsupported)
            | Self::DateTime(unsupported)
            | Self::BigNumeric(unsupported)
            | Self::Numeric(unsupported) => match *unsupported {},
        }

        Ok(())
    }
}
