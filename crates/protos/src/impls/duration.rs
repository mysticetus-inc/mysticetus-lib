#![allow(dead_code)]
use timestamp::Duration;

use crate::protobuf;

impl From<protobuf::Duration> for Duration {
    fn from(p: protobuf::Duration) -> Self {
        Duration::new(p.seconds, p.nanos)
    }
}

impl From<Duration> for protobuf::Duration {
    fn from(d: Duration) -> Self {
        protobuf::Duration {
            seconds: d.whole_seconds(),
            nanos: d.subsec_nanoseconds(),
        }
    }
}

pub fn serialize<S>(p: &protobuf::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serde::Serialize::serialize(&Duration::from(*p).as_seconds_f64(), serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<protobuf::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let dur = <Duration as serde::Deserialize>::deserialize(deserializer)?;
    Ok(dur.into())
}

pub mod opt {
    use super::*;

    pub fn serialize<S>(p: &Option<protobuf::Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *p {
            Some(dur) => serializer.serialize_some(&Duration::from(dur).as_seconds_f64()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<protobuf::Duration>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<Duration> as serde::Deserialize>::deserialize(deserializer)? {
            Some(ts) => Ok(Some(ts.into())),
            None => Ok(None),
        }
    }
}
