#![allow(dead_code)]

use ::timestamp::Timestamp;

use crate::protobuf;

impl From<protobuf::Timestamp> for Timestamp {
    fn from(ts: protobuf::Timestamp) -> Self {
        Timestamp::from_seconds(ts.seconds).add_nanos(ts.nanos as _)
    }
}

impl From<Timestamp> for protobuf::Timestamp {
    fn from(ts: Timestamp) -> protobuf::Timestamp {
        protobuf::Timestamp {
            seconds: ts.as_seconds(),
            nanos: ts.subsec_nanos(),
        }
    }
}

pub fn serialize<S>(p: &protobuf::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serde::Serialize::serialize(&Timestamp::from(*p), serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<protobuf::Timestamp, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ts = <Timestamp as serde::Deserialize>::deserialize(deserializer)?;
    Ok(ts.into())
}

pub mod opt {
    use super::*;

    pub fn serialize<S>(p: &Option<protobuf::Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *p {
            Some(ts) => serializer.serialize_some(&Timestamp::from(ts)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<protobuf::Timestamp>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<Timestamp> as serde::Deserialize>::deserialize(deserializer)? {
            Some(ts) => Ok(Some(ts.into())),
            None => Ok(None),
        }
    }
}
