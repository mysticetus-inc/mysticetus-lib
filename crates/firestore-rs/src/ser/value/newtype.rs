use protos::firestore;

use super::ValueSerializer;
use crate::error::SerError;
use crate::ser::SerializerOptions;
use crate::ser::timestamp::FirestoreTimestamp;
use crate::ser::value::SerializedValueKind;

pub(super) struct MaybeTimestampSerializer<N: SerializerOptions> {
    inner: ValueSerializer<N>,
}

impl<N: SerializerOptions> MaybeTimestampSerializer<N> {
    #[inline]
    pub(super) fn new(inner: ValueSerializer<N>) -> Self {
        Self { inner }
    }
}

macro_rules! defer_to_inner_serializer {
    ($(
        $method:ident(
            $(
                $var:ident: $var_ty:ty
            ),* $(,)?
        ) -> $ok_ty:ident
    ),* $(,)?) => {

        $(
            #[inline]
            fn $method(self, $($var: $var_ty,)*) -> Result<Self::$ok_ty, Self::Error> {
                self.inner.$method($($var,)*)
            }
        )*
    };
}

impl<N: SerializerOptions> serde::Serializer for MaybeTimestampSerializer<N> {
    type Ok = SerializedValueKind<N::FieldTransform>;
    type Error = SerError;

    type SerializeMap = <ValueSerializer<N> as serde::Serializer>::SerializeMap;
    type SerializeSeq = <ValueSerializer<N> as serde::Serializer>::SerializeSeq;
    type SerializeTuple = <ValueSerializer<N> as serde::Serializer>::SerializeTuple;
    type SerializeStruct = <ValueSerializer<N> as serde::Serializer>::SerializeStruct;
    type SerializeTupleStruct = <ValueSerializer<N> as serde::Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <ValueSerializer<N> as serde::Serializer>::SerializeTupleVariant;
    type SerializeStructVariant = <ValueSerializer<N> as serde::Serializer>::SerializeStructVariant;

    // we really only care about this method (and to a lesser extent serialize_some),
    // since the newtype FirestoreTimestamp serializes as an i128 as a number of
    // nano seconds. Otherwise, all methods defer to the inner ValueSerializer
    #[inline]
    fn serialize_i128(self, nanos: i128) -> Result<Self::Ok, Self::Error> {
        let FirestoreTimestamp(proto_ts) = FirestoreTimestamp::from_nanos(nanos);
        Ok(Some(firestore::Value {
            value_type: Some(firestore::value::ValueType::TimestampValue(proto_ts)),
        }))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    defer_to_inner_serializer! {
        serialize_bool(v: bool) -> Ok,
        serialize_i8(v: i8) -> Ok,
        serialize_i16(v: i16) -> Ok,
        serialize_i32(v: i32) -> Ok,
        serialize_i64(v: i64) -> Ok,
        serialize_u8(v: u8) -> Ok,
        serialize_u16(v: u16) -> Ok,
        serialize_u32(v: u32) -> Ok,
        serialize_u64(v: u64) -> Ok,
        serialize_u128(v: u128) -> Ok,
        serialize_f32(v: f32) -> Ok,
        serialize_f64(v: f64) -> Ok,
        serialize_char(v: char) -> Ok,
        serialize_str(v: &str) -> Ok,
        serialize_bytes(v: &[u8]) -> Ok,
        serialize_none() -> Ok,
        serialize_unit() -> Ok,
        serialize_unit_struct(name: &'static str) -> Ok,
        serialize_unit_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
        ) -> Ok,
        serialize_seq(len: Option<usize>) -> SerializeSeq,
        serialize_tuple(len: usize) -> SerializeSeq,
        serialize_tuple_struct(
            name: &'static str,
            len: usize,
        ) -> SerializeTupleStruct,
        serialize_tuple_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> SerializeTupleVariant,
        serialize_map(len: Option<usize>) -> SerializeMap,
        serialize_struct(
            name: &'static str,
            len: usize,
        ) -> SerializeStruct,
        serialize_struct_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> SerializeStructVariant,
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        self.inner.collect_str(value)
    }
}

pub(super) struct MaybeReferenceSerializer<N: SerializerOptions> {
    inner: ValueSerializer<N>,
}

impl<N: SerializerOptions> MaybeReferenceSerializer<N> {
    #[inline]
    pub(super) fn new(inner: ValueSerializer<N>) -> Self {
        Self { inner }
    }
}

macro_rules! defer_to_inner_serializer {
    ($(
        $method:ident(
            $(
                $var:ident: $var_ty:ty
            ),* $(,)?
        ) -> $ok_ty:ident
    ),* $(,)?) => {

        $(
            #[inline]
            fn $method(self, $($var: $var_ty,)*) -> Result<Self::$ok_ty, Self::Error> {
                self.inner.$method($($var,)*)
            }
        )*
    };
}

impl<N: SerializerOptions> serde::Serializer for MaybeReferenceSerializer<N> {
    type Ok = SerializedValueKind<N::FieldTransform>;
    type Error = SerError;

    type SerializeMap = <ValueSerializer<N> as serde::Serializer>::SerializeMap;
    type SerializeSeq = <ValueSerializer<N> as serde::Serializer>::SerializeSeq;
    type SerializeTuple = <ValueSerializer<N> as serde::Serializer>::SerializeTuple;
    type SerializeStruct = <ValueSerializer<N> as serde::Serializer>::SerializeStruct;
    type SerializeTupleStruct = <ValueSerializer<N> as serde::Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <ValueSerializer<N> as serde::Serializer>::SerializeTupleVariant;
    type SerializeStructVariant = <ValueSerializer<N> as serde::Serializer>::SerializeStructVariant;

    fn serialize_str(self, maybe_reference: &str) -> Result<Self::Ok, Self::Error> {
        // do some super basic validation that this might actually be a reference, otherwise
        // just defer to normal string serialization.

        let parts = maybe_reference.split('/').count();

        // we expect at least 6 path components, and there needs to be a multiple of 2
        // to refer to a document (otherwise this might be a collection reference, which
        // isnt valid in this case)
        if 6 <= parts && parts % 2 == 0 {
            Ok(Some(firestore::Value {
                value_type: Some(firestore::value::ValueType::ReferenceValue(
                    maybe_reference.to_owned(),
                )),
            }))
        } else {
            self.inner.serialize_str(maybe_reference)
        }
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    defer_to_inner_serializer! {
        serialize_bool(v: bool) -> Ok,
        serialize_i8(v: i8) -> Ok,
        serialize_i16(v: i16) -> Ok,
        serialize_i32(v: i32) -> Ok,
        serialize_i64(v: i64) -> Ok,
        serialize_i128(v: i128) -> Ok,
        serialize_u8(v: u8) -> Ok,
        serialize_u16(v: u16) -> Ok,
        serialize_u32(v: u32) -> Ok,
        serialize_u64(v: u64) -> Ok,
        serialize_u128(v: u128) -> Ok,
        serialize_f32(v: f32) -> Ok,
        serialize_f64(v: f64) -> Ok,
        serialize_char(v: char) -> Ok,
        serialize_bytes(v: &[u8]) -> Ok,
        serialize_none() -> Ok,
        serialize_unit() -> Ok,
        serialize_unit_struct(name: &'static str) -> Ok,
        serialize_unit_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
        ) -> Ok,
        serialize_seq(len: Option<usize>) -> SerializeSeq,
        serialize_tuple(len: usize) -> SerializeSeq,
        serialize_tuple_struct(
            name: &'static str,
            len: usize,
        ) -> SerializeTupleStruct,
        serialize_tuple_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> SerializeTupleVariant,
        serialize_map(len: Option<usize>) -> SerializeMap,
        serialize_struct(
            name: &'static str,
            len: usize,
        ) -> SerializeStruct,
        serialize_struct_variant(
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> SerializeStructVariant,
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        self.inner.collect_str(value)
    }
}
