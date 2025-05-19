use std::borrow::Cow;

use bytes::{BufMut, Bytes};
use prost::Message;
use serde::Serialize;

pub mod range;

pub use range::{Range, RangeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unsupported {}

#[derive(Debug, Clone)]
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
    Repeated(Unsupported),
    Record(Unsupported),
}

impl<'a, T: Serialize + ?Sized + 'a> Value<'a, T> {
    pub fn encode<B: BufMut + ?Sized>(&self, buf: &mut B) -> serde_json::Result<()> {
        match self {
            Self::Bytes(bytes) => bytes.encode_raw(&mut buf),
            Self::Bool(b) => b.encode_raw(&mut buf),
            Self::Date(date) => date
                .days_since(timestamp::Date::UNIX_EPOCH)
                .encode_raw(&mut buf),
            Self::Timestamp(ts) => protos::protobuf::Timestamp::from(*ts).encode_raw(&mut buf),
            Self::Integer(int) => int.encode_raw(&mut buf),
            Self::Range(range) => range.encode(&mut buf),
            Self::Interval(interval) => {
                protos::protobuf::Duration::from(*interval).encode_raw(&mut buf)
            }
            Self::Time(time) => time.to_string().encode_raw(&mut buf),
            Self::String(str) => str.encode_raw(&mut buf),
            Self::Json(json) => serde_json::to_string(json)?.encode_raw(&mut buf),
            Self::Geography() => todo!(),
            Self::Record(unsupported)
            | Self::Repeated(unsupported)
            | Self::DateTime(unsupported)
            | Self::BigNumeric(unsupported)
            | Self::Numeric(unsupported) => match *unsupported {},
        }

        Ok(())
    }
}
