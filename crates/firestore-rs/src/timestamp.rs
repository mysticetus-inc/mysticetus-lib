//! Serde compat types for serializing as a firestore timestamp type.

use serde::Deserialize;

// we encode protos::protobuf::Timestamp as a flat byte array,
// containing the NE bytes of the i64 seconds, followed by the NE
// bytes of the i32 subsec nanos.
const SIZE: usize = std::mem::size_of::<i64>() + std::mem::size_of::<i32>();

/// Concrete new-type that a serializer can use to enforce serialization as a
/// firestore timestamp.
///
/// We serialize the protobuf Timestamp to a flat byte array, that way
/// we can use a simpler detection method when serializing
/// (i.e serializing as bytes)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct FirestoreTimestamp(pub(crate) [u8; SIZE]);

impl FirestoreTimestamp {
    pub(crate) const MARKER: &str = "__firestore_timestamp__";

    pub(crate) fn encode(ts: protos::protobuf::Timestamp) -> Self {
        let secs = ts.seconds.to_ne_bytes();
        let nanos = ts.nanos.to_ne_bytes();

        Self([
            secs[0], secs[1], secs[2], secs[3], secs[4], secs[5], secs[6], secs[7], nanos[0],
            nanos[1], nanos[2], nanos[3],
        ])
    }

    pub(crate) fn decode(&self) -> protos::protobuf::Timestamp {
        protos::protobuf::Timestamp {
            seconds: self.seconds(),
            nanos: self.subsec_nanos(),
        }
    }

    pub(crate) fn try_serialize<T: serde::Serialize + ?Sized>(
        value: &T,
    ) -> Result<protos::protobuf::Timestamp, InvalidTimestamp> {
        value
            .serialize(FirestoreTimestampSerializer)
            .map(|ts| ts.decode())
    }
}

impl serde::Serialize for FirestoreTimestamp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        struct SerializeAsBytes<'a>(&'a [u8]);

        impl serde::Serialize for SerializeAsBytes<'_> {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_bytes(self.0)
            }
        }

        serializer.serialize_newtype_struct(Self::MARKER, &SerializeAsBytes(&self.0))
    }
}

impl<'de> serde::Deserialize<'de> for FirestoreTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> serde::de::Visitor<'vde> for Visitor {
            type Value = [u8; SIZE];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an encoded timestamp as a {SIZE} byte array")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match <&[u8] as TryInto<&[u8; SIZE]>>::try_into(v) {
                    Ok(array) => Ok(*array),
                    Err(_) => Err(serde::de::Error::invalid_length(v.len(), &self)),
                }
            }
        }

        deserializer
            .deserialize_newtype_struct(Self::MARKER, Visitor)
            .map(Self)
    }
}

/// Trait that can be implemented by external types that
/// define conversion to and from the protobuf repr for a Timestamp.
pub trait AsTimestamp: Sized {
    type Error: std::error::Error;

    fn seconds(&self) -> i64;
    fn subsec_nanos(&self) -> i32;

    fn from_parts(seconds: i64, subsec_nanos: i32) -> Result<Self, Self::Error>;
}

impl AsTimestamp for FirestoreTimestamp {
    type Error = std::convert::Infallible;

    #[inline]
    fn seconds(&self) -> i64 {
        i64::from_ne_bytes([
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7],
        ])
    }

    #[inline]
    fn subsec_nanos(&self) -> i32 {
        i32::from_ne_bytes([self.0[8], self.0[9], self.0[10], self.0[11]])
    }

    #[inline]
    fn from_parts(seconds: i64, nanos: i32) -> Result<Self, Self::Error> {
        Ok(Self::encode(protos::protobuf::Timestamp { seconds, nanos }))
    }
}

impl AsTimestamp for timestamp::Timestamp {
    type Error = std::convert::Infallible;

    #[inline]
    fn seconds(&self) -> i64 {
        self.as_seconds()
    }

    #[inline]
    fn subsec_nanos(&self) -> i32 {
        self.subsec_nanos()
    }

    #[inline]
    fn from_parts(seconds: i64, nanos: i32) -> Result<Self, Self::Error> {
        Ok(Self::from(protos::protobuf::Timestamp { seconds, nanos }))
    }
}

#[inline]
pub fn serialize<T, S>(timestamp: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsTimestamp,
    S: serde::Serializer,
{
    serde::Serialize::serialize(
        &FirestoreTimestamp::encode(protos::protobuf::Timestamp {
            seconds: timestamp.seconds(),
            nanos: timestamp.subsec_nanos(),
        }),
        serializer,
    )
}

#[inline]
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: AsTimestamp,
    D: serde::Deserializer<'de>,
{
    let protos::protobuf::Timestamp { seconds, nanos } =
        FirestoreTimestamp::deserialize(deserializer)?.decode();

    T::from_parts(seconds, nanos).map_err(serde::de::Error::custom)
}

pub mod optional {
    use serde::Deserialize;

    use super::{AsTimestamp, FirestoreTimestamp};

    #[inline]
    pub fn serialize<T, S>(timestamp: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsTimestamp,
        S: serde::Serializer,
    {
        match timestamp {
            Some(ts) => serializer.serialize_some(&FirestoreTimestamp::encode(
                protos::protobuf::Timestamp {
                    seconds: ts.seconds(),
                    nanos: ts.subsec_nanos(),
                },
            )),
            None => serializer.serialize_none(),
        }
    }

    #[inline]
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: AsTimestamp,
        D: serde::Deserializer<'de>,
    {
        match Option::<FirestoreTimestamp>::deserialize(deserializer)? {
            None => Ok(None),
            Some(ts) => {
                let protos::protobuf::Timestamp { seconds, nanos } = ts.decode();

                T::from_parts(seconds, nanos)
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}

pub(crate) struct FirestoreTimestampSerializer;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InvalidTimestamp {
    #[error("mismatched byte array size, expected {SIZE} bytes")]
    EncodedSizeMismatch,
    #[error("expected bytes, got the wrong type")]
    WrongType,
}

impl serde::ser::Error for InvalidTimestamp {
    fn custom<T>(_: T) -> Self
    where
        T: std::fmt::Display,
    {
        if cfg!(debug_assertions) {
            panic!(
                "<firestore_rs::ser::timestamp::InvalidTimestamp as serde::ser::Error>::custom \
                 should never be called"
            );
        }

        Self::WrongType
    }
}

macro_rules! impl_wrong_type_methods {
    (
        $($name:ident($arg_ty:ty)),* $(,)?
    ) => {
        $(
            #[inline]
            fn $name(self, _: $arg_ty) -> Result<Self::Ok, Self::Error> {
                Err(InvalidTimestamp::WrongType)
            }
        )*
    };
}

impl serde::Serializer for FirestoreTimestampSerializer {
    type Ok = FirestoreTimestamp;
    type Error = InvalidTimestamp;

    type SerializeSeq = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeTuple = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeTupleStruct = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeTupleVariant = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeMap = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeStruct = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;
    type SerializeStructVariant = serde::ser::Impossible<FirestoreTimestamp, InvalidTimestamp>;

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match <&[u8] as TryInto<&[u8; SIZE]>>::try_into(v) {
            Ok(array) => Ok(FirestoreTimestamp(*array)),
            Err(_) => Err(InvalidTimestamp::EncodedSizeMismatch),
        }
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    impl_wrong_type_methods! {
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(InvalidTimestamp::WrongType)
    }
}
