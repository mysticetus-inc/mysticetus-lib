//! A serializer to create [`firestore::Value`] from [`Serialize`]-able types.
use std::cell::RefCell;
use std::marker::PhantomData;

use protos::firestore::value::ValueType;
use protos::firestore::{self, ArrayValue};
use serde::ser::{Serialize, Serializer};

pub(crate) mod geo;
// mod newtype;
pub(crate) mod timestamp;

mod map;
mod seq;

use crate::error::SerError;
use crate::ser::field_transform::Transform;
use crate::ser::{MapSerializerKind, WriteKind};
use crate::timestamp::FirestoreTimestamp;
use crate::{Reference, transform};

/// Handles serializing [`Serialize`]-able types into a [`firestore::Value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueSerializer<'a, N, T: Transform> {
    container: T::Container<'a>,
    _marker: PhantomData<fn(N)>,
}

impl<'a, W: WriteKind, T: Transform<Container<'a>: Default>> Default for ValueSerializer<'a, W, T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<'a, W: WriteKind, T: Transform> ValueSerializer<'a, W, T> {
    pub(crate) const fn new(container: T::Container<'a>) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    pub fn reborrow(&mut self) -> ValueSerializer<'_, W, T> {
        ValueSerializer {
            container: T::reborrow(&mut self.container),
            _marker: PhantomData,
        }
    }

    pub fn serialize(
        &mut self,
        value: &(impl serde::Serialize + ?Sized),
    ) -> Result<SerializedValueKind<T>, SerError>
    where
        for<'b> &'b mut Self: serde::Serializer<Ok = SerializedValueKind<T>, Error = SerError>,
    {
        value.serialize(self)
    }
}

impl<W: WriteKind> ValueSerializer<'_, W, std::convert::Infallible> {
    pub(crate) fn seq<V>(mut self, raw_values: &[&V]) -> Result<firestore::Value, SerError>
    where
        V: Serialize,
    {
        let mut values = Vec::with_capacity(raw_values.len());

        for result in raw_values
            .iter()
            .map(move |value| value.serialize(&mut self))
        {
            match result? {
                SerializedValueKind::Value(ValueType::NullValue(_)) if W::MERGE => (),
                SerializedValueKind::Value(ValueType::NullValue(_)) => {
                    values.push(super::null_value())
                }
                SerializedValueKind::Value(value_type) => values.push(firestore::Value {
                    value_type: Some(value_type),
                }),
            }
        }

        Ok(firestore::Value {
            value_type: Some(ValueType::ArrayValue(ArrayValue { values })),
        })
    }
}

pub enum SerializedValueKind<Transform> {
    Value(ValueType),
    Transform(Transform),
}

impl<'a, 'b, W: WriteKind, T: Transform> Serializer for &'b mut ValueSerializer<'a, W, T>
where
    map::ValueMapSerializer<'b, W, T>:
        MapSerializerKind<ValueSerializer<'b, W, T>, Output = SerializedValueKind<T>>,
{
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    type SerializeSeq = seq::ValueSeqSerializer<'b, W, T>;
    type SerializeTuple = seq::ValueSeqSerializer<'b, W, T>;
    type SerializeTupleStruct = seq::ValueSeqSerializer<'b, W, T>;
    type SerializeTupleVariant = seq::ValueSeqSerializer<'b, W, T>;

    type SerializeMap = map::ValueMapSerializer<'b, W, T>;
    type SerializeStruct = map::ValueMapSerializer<'b, W, T>;
    type SerializeStructVariant = map::ValueMapSerializer<'b, W, T>;

    fn serialize_bool(self, boolean: bool) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::BooleanValue(boolean)))
    }

    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::BytesValue(
            bytes.to_vec().into(),
        )))
    }

    fn serialize_char(self, c: char) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::StringValue(
            String::from(c),
        )))
    }

    fn serialize_f32(self, float: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(float as f64)
    }

    fn serialize_f64(self, float: f64) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::DoubleValue(float)))
    }

    // Redirect all integer types to this master function
    fn serialize_i64(self, int: i64) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::IntegerValue(int)))
    }

    fn serialize_i8(self, int: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_i16(self, int: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_i32(self, int: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u8(self, int: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u16(self, int: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u32(self, int: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u64(self, int: u64) -> Result<Self::Ok, Self::Error> {
        match int.try_into().ok() {
            Some(casted_int) => self.serialize_i64(casted_int),
            None => self.serialize_str(&int.to_string()),
        }
    }

    serde::serde_if_integer128! {
        fn serialize_i128(self, int: i128) -> Result<Self::Ok, Self::Error> {
            match int.try_into().ok() {
                Some(casted_int) => self.serialize_i64(casted_int),
                None => self.serialize_str(&int.to_string()),
            }
        }

        fn serialize_u128(self, int: u128) -> Result<Self::Ok, Self::Error> {
            match int.try_into().ok() {
                Some(casted_int) => self.serialize_i64(casted_int),
                None => self.serialize_str(&int.to_string()),
            }
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(map::ValueMapSerializer::new_with_len(len, self.reborrow()))
    }

    fn serialize_newtype_struct<V>(
        self,
        name: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: Serialize + ?Sized,
    {
        match name {
            transform::Increment::MARKER => T::increment(value).map(SerializedValueKind::Transform),
            transform::Maximum::MARKER => T::maximum(value).map(SerializedValueKind::Transform),
            transform::Minimum::MARKER => T::minimum(value).map(SerializedValueKind::Transform),
            transform::ArrayUnion::MARKER => {
                T::append_missing_elements::<V, W>(value).map(SerializedValueKind::Transform)
            }
            transform::RemoveFromArray::MARKER => {
                T::remove_all_from_array::<V, W>(value).map(SerializedValueKind::Transform)
            }
            FirestoreTimestamp::MARKER => FirestoreTimestamp::try_serialize(value)
                .map(|ts| SerializedValueKind::Value(ValueType::TimestampValue(ts)))
                .map_err(serde::ser::Error::custom),
            Reference::MARKER => {
                Reference::try_serialize::<W>(value).map(SerializedValueKind::Value)
            }
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<V>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::NullValue(0)))
    }

    fn serialize_some<V>(self, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(seq::ValueSeqSerializer::new(len))
    }

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::StringValue(
            s.to_owned(),
        )))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        if name == transform::ServerTimestamp::MARKER {
            let transform = T::server_timestamp()?;
            Ok(SerializedValueKind::Transform(transform))
        } else {
            self.serialize_str(name)
        }
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn collect_str<V>(self, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + std::fmt::Display,
    {
        thread_local! {
            static COLLECT_STR_BUF: RefCell<String> = RefCell::new(String::with_capacity(256));
        }

        COLLECT_STR_BUF.with_borrow_mut(|str_buf| {
            str_buf.clear();
            std::fmt::write(str_buf, format_args!("{value}"))
                .map_err(<SerError as serde::ser::Error>::custom)?;

            self.serialize_str(str_buf.as_str())
        })
    }
}
