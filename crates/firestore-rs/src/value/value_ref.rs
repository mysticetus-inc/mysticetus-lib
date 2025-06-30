use protos::r#type::LatLng;
use timestamp::Timestamp;

use super::array::ArrayRef;
use super::map::MapRef;
use super::reference::Reference;

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
    Reference(&'a Reference),
    Array(ArrayRef<'a>),
    Map(MapRef<'a>),
}

impl<'a> ValueRef<'a> {
    pub(crate) fn from_proto_type_ref(value: &'a protos::firestore::value::ValueType) -> Self {
        use protos::firestore::value::ValueType::*;

        match value {
            NullValue(_) => Self::Null,
            TimestampValue(ts) => Self::Timestamp((*ts).into()),
            BooleanValue(b) => Self::Bool(*b),
            BytesValue(bytes) => Self::Bytes(bytes),
            GeoPointValue(gp) => Self::GeoPoint(*gp),
            StringValue(s) => Self::String(s),
            IntegerValue(i) => Self::Integer(*i),
            DoubleValue(f) => Self::Double(*f),
            ReferenceValue(refer) => Self::Reference(Reference::new(refer.as_str())),
            ArrayValue(array) => Self::Array(ArrayRef::from_values(&array.values)),
            MapValue(map) => Self::Map(MapRef::from_fields(&map.fields)),
        }
    }

    pub(crate) fn from_proto_ref(value: &'a protos::firestore::Value) -> Self {
        value
            .value_type
            .as_ref()
            .map(Self::from_proto_type_ref)
            .unwrap_or(Self::Null)
    }

    pub(crate) fn ord_cmp(&self, _other: ValueRef<'_>) -> std::cmp::Ordering {
        todo!()
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

/*
impl PartialOrd for ValueRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {}
}

impl Ord for ValueRef<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Copied from:
        // https://github.com/googleapis/google-cloud-dotnet/blob/f76e11a2f3a7403cb9199b72b2f5e1a303c9d50d/apis/Google.Cloud.Firestore/Google.Cloud.Firestore/ValueComparer.cs#L29
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        enum TypeOrder {
            Null,
            Boolean,
            Number,
            Timestamp,
            String,
            Blob,
            Ref,
            GeoPoint,
            Array,
            Object,
        }

        impl TypeOrder {
            fn from_value(value: &ValueRef<'_>) -> Self {
                match value {
                    ValueRef::Null => Self::Null,
                    ValueRef::Map(_) => Self::Object,
                    ValueRef::Bool(_) => Self::Boolean,
                    ValueRef::Array(_) => Self::Array,
                    ValueRef::Timestamp(_) => Self::Timestamp,
                    ValueRef::Bytes(_) => Self::Blob,
                    ValueRef::Integer(_) | ValueRef::Double(_) => Self::Number,
                    ValueRef::String(_) => Self::String,
                    ValueRef::GeoPoint(_) => Self::GeoPoint,
                    ValueRef::Reference(_) => Self::Ref,
                }
            }
        }

        fn insane_dotnet_double_ord(a: f64, b: f64) -> std::cmp::Ordering {
            use std::num::FpCategory::*;

            match (a.classify(), b.classify()) {
                // if both are finite numbers, use total_cmp
                (Normal | Subnormal | Zero, Normal | Subnormal | Zero) => a.total_cmp(&b),
                (Nan, Nan) => std::cmp::Ordering::Equal,
                (Infinite, Infinite) => match (a.is_sign_positive(), b.is_sign_positive()) {
                    (true, true) | (false, false) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                },
                (Nan, Normal | Subnormal | Zero | Infinite) => std::cmp::Ordering::Less,
                (Normal | Subnormal | Zero | Infinite, Nan) => std::cmp::Ordering::Greater,
                (Infinite, Subnormal | Zero) => a.total_cmp(&b),
            }
        }

        use ValueRef::*;

        match (self, other) {
            // all nulls are equal according to the reference c# impl
            (Null, Null) => std::cmp::Ordering::Equal,
            (String(a), String(b)) => a.cmp(b),
            (Bytes(a), Bytes(b)) => a.cmp(b),
            (Reference(a), Reference(b)) => a.cmp(b),
            (Bool(a), Bool(b)) => a.cmp(b),
            (GeoPoint(a), GeoPoint(b)) => {} /* _ => TypeOrder::from_value(self).cmp(&
                                              * TypeOrder::from_value(other)), */
        }
    }
}
*/
