#![allow(dead_code)] // while in dev

use bytes::Bytes;
use protos::firestore;
use protos::firestore::value::ValueType;
use protos::protobuf::NullValue;
use protos::r#type::LatLng;
use timestamp::Timestamp;

use super::reference::ReferenceRef;
use super::{Array, Map, Reference, ValueRef};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Double(f64),
    Timestamp(Timestamp),
    String(String),
    Bytes(Bytes),
    Reference(Reference),
    GeoPoint(LatLng),
    Array(Array),
    Map(Map),
}

impl From<firestore::Value> for Value {
    fn from(value: firestore::Value) -> Self {
        Self::from_proto_value(value)
    }
}

impl Value {
    #[inline]
    pub(crate) fn into_proto_value(self) -> firestore::Value {
        firestore::Value {
            value_type: Some(match self {
                Self::Null => ValueType::NullValue(NullValue::NullValue as i32),
                Self::Bool(b) => ValueType::BooleanValue(b),
                Self::Integer(int) => ValueType::IntegerValue(int),
                Self::Double(fl) => ValueType::DoubleValue(fl),
                Self::Timestamp(ts) => ValueType::TimestampValue(ts.into()),
                Self::String(s) => ValueType::StringValue(s),
                Self::Bytes(bytes) => ValueType::BytesValue(bytes),
                Self::Reference(refer) => ValueType::ReferenceValue(refer.0),
                Self::GeoPoint(geo) => ValueType::GeoPointValue(geo),
                Self::Array(a) => return a.into_proto_value(),
                Self::Map(m) => return m.into_proto_value(),
            }),
        }
    }

    pub fn as_ref(&self) -> ValueRef<'_> {
        match self {
            Self::Null => ValueRef::Null,
            Self::Bool(b) => ValueRef::Bool(*b),
            Self::Bytes(bytes) => ValueRef::Bytes(bytes),
            Self::String(s) => ValueRef::String(s),
            Self::Double(d) => ValueRef::Double(*d),
            Self::Integer(i) => ValueRef::Integer(*i),
            Self::Reference(refer) => ValueRef::Reference(ReferenceRef(&refer.0)),
            Self::Timestamp(ts) => ValueRef::Timestamp(*ts),
            Self::GeoPoint(gp) => ValueRef::GeoPoint(*gp),
            Self::Map(map) => ValueRef::Map(map.as_ref()),
            Self::Array(arr) => ValueRef::Array(arr.as_ref()),
        }
    }

    #[inline]
    pub(crate) fn from_proto_value(value: firestore::Value) -> Self {
        value
            .value_type
            .map(Self::from_proto_value_type)
            .unwrap_or(Self::Null)
    }

    #[inline]
    pub(crate) fn from_proto_value_type(value_type: ValueType) -> Self {
        use ValueType::*;
        match value_type {
            NullValue(_) => Value::Null,
            BooleanValue(b) => Value::Bool(b),
            IntegerValue(int) => Value::Integer(int),
            DoubleValue(fl) => Value::Double(fl),
            TimestampValue(ts) => Value::Timestamp(ts.into()),
            StringValue(s) => Value::String(s),
            BytesValue(b) => Value::Bytes(b),
            ReferenceValue(refer) => Value::Reference(Reference(refer)),
            GeoPointValue(pt) => Value::GeoPoint(pt),
            MapValue(map) => Value::Map(Map::from_proto_value(map)),
            ArrayValue(arr) => Value::Array(Array::from_proto_value(arr)),
        }
    }

    #[cfg(test)]
    pub fn rand<R: rand::Rng>(rng: &mut R, avail_nesting: usize, allow_nested: bool) -> Self {
        fn gen_timestamp<R: rand::Rng>(rng: &mut R) -> timestamp::Timestamp {
            thread_local!(static NOW: f64 = timestamp::Timestamp::now().as_seconds_f64());
            const START: f64 = timestamp::Timestamp::UNIX_EPOCH.as_seconds_f64();

            let now = NOW.with(|ts| *ts);

            let ts = rng.random_range(START..=now);

            timestamp::Timestamp::from_seconds_f64_checked(ts).unwrap()
        }

        fn gen_geo<R: rand::Rng>(rng: &mut R) -> crate::LatLng {
            let latitude = rng.random_range(-90.0..=90.0);
            let longitude = rng.random_range(-180.0..=180.0);

            crate::LatLng {
                latitude,
                longitude,
            }
        }

        fn gen_bytes<R: rand::Rng>(rng: &mut R) -> Bytes {
            let len = rng.random_range(0_usize..128);
            let mut dst = bytes::BytesMut::with_capacity(len);

            let bytes =
                <&mut R as rand::Rng>::sample_iter::<u8, _>(rng, rand::distr::StandardUniform)
                    .take(len);

            dst.extend(bytes);
            dst.freeze()
        }

        let mut max = 7;

        if avail_nesting == 0 {
            max += 2;
        }
        let nesting = avail_nesting.checked_sub(1).unwrap_or_default();

        match rng.random_range(0..=max) {
            0 => Self::Null,
            1 => Self::Bool(rng.random()),
            2 => Self::Integer(rng.random()),
            3 => Self::Double(rng.random()),
            4 => Self::Timestamp(gen_timestamp(rng)),
            5 => Self::String(super::gen_string(rng)),
            6 => Self::Bytes(gen_bytes(rng)),
            7 => Self::GeoPoint(gen_geo(rng)),
            8 if allow_nested => Self::Array(Array::rand(rng, nesting, allow_nested)),
            8 | 9 => Self::Map(Map::rand(rng, nesting, allow_nested)),
            _ => unreachable!("should never be 10+"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(super::de::ValueVisitor)
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}
