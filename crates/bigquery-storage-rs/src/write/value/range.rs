use bytes::BufMut;
use prost::Message;
use timestamp::{Date, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeKind {
    Date(Range<Date>),
    DateTime(super::Unsupported),
    Timestamp(Range<Timestamp>),
}

impl RangeKind {
    pub(super) fn encode(&self, buf: &mut impl BufMut) {
        match self {
            Self::Date(date_range) => date_range.as_message().encode_raw(buf),
            Self::Timestamp(ts_range) => ts_range.as_message().encode_raw(buf),
            Self::DateTime(unsupported) => match *unsupported {},
        }
    }
}

impl Range<Date> {
    fn as_message(&self) -> DateRange {
        #[inline]
        const fn convert_to_days_since_unix_epoch(date: Date) -> i32 {
            date.days_since(Date::UNIX_EPOCH)
        }

        DateRange {
            start: self.start.map(convert_to_days_since_unix_epoch),
            end: self.end.map(convert_to_days_since_unix_epoch),
        }
    }
}

impl Range<Timestamp> {
    fn as_message(&self) -> TimestampRange {
        TimestampRange {
            start: self.start.map(Into::into),
            end: self.end.map(Into::into),
        }
    }
}

/// Dedicated, private types for serializing as the expected protobuf types

#[derive(Debug, prost::Message)]
struct DateRange {
    #[prost(int32, optional)]
    start: Option<i32>,
    #[prost(int32, optional)]
    end: Option<i32>,
}

#[derive(Debug, prost::Message)]
struct TimestampRange {
    #[prost(message, optional)]
    start: Option<protos::protobuf::Timestamp>,
    #[prost(message, optional)]
    end: Option<protos::protobuf::Timestamp>,
}
