use protos::r#type::LatLng;
use timestamp::Timestamp;

use super::array::ArrayRef;
use super::map::MapRef;
use super::reference::ReferenceRef;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueRef<'a> {
    Null,
    Bool(bool),
    Integer(i64),
    Double(f64),
    Timestamp(Timestamp),
    String(&'a str),
    Bytes(&'a [u8]),
    GeoPoint(LatLng),
    Reference(ReferenceRef<'a>),
    Array(ArrayRef<'a>),
    Map(MapRef<'a>),
}

impl<'a> ValueRef<'a> {
    pub(super) fn from_proto_ref(value: &'a protos::firestore::Value) -> Self {
        use protos::firestore::value::ValueType::*;

        match value.value_type {
            None | Some(NullValue(_)) => Self::Null,
            Some(TimestampValue(ts)) => Self::Timestamp(ts.into()),
            Some(BooleanValue(b)) => Self::Bool(b),
            Some(BytesValue(ref bytes)) => Self::Bytes(bytes),
            Some(GeoPointValue(gp)) => Self::GeoPoint(gp),
            Some(StringValue(ref s)) => Self::String(s),
            Some(IntegerValue(i)) => Self::Integer(i),
            Some(DoubleValue(f)) => Self::Double(f),
            Some(ReferenceValue(ref refer)) => Self::Reference(ReferenceRef(refer)),
            Some(ArrayValue(ref array)) => Self::Array(ArrayRef::from_values(&array.values)),
            Some(MapValue(ref map)) => Self::Map(MapRef::from_fields(&map.fields)),
        }
    }
}

impl serde::Serialize for ValueRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Bytes(bytes) => serializer.serialize_bytes(bytes),
            Self::Timestamp(ts) => ts.serialize(serializer),
            Self::Double(d) => serializer.serialize_f64(*d),
            Self::Integer(i) => serializer.serialize_i64(*i),
            Self::String(s) => serializer.serialize_str(s),
            Self::GeoPoint(gp) => [gp.longitude, gp.latitude].serialize(serializer),
            Self::Reference(refer) => refer.serialize(serializer),
            Self::Array(array) => array.serialize(serializer),
            Self::Map(map) => map.serialize(serializer),
        }
    }
}
