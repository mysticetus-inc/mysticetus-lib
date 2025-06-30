use std::convert::Infallible;
use std::marker::PhantomData;

use protos::firestore::value::ValueType::{self, NullValue};
use protos::firestore::{self, ArrayValue};
use serde::{Serialize, ser};

use crate::error::SerError;
use crate::ser::field_transform::NoFieldTransforms;
use crate::ser::value::SerializedValueKind;
use crate::ser::{ValueSerializer, WriteKind};

pub struct ValueSeqSerializer<'a, W, T> {
    inner: ValueSerializer<'a, W, Infallible>,
    values: Vec<firestore::Value>,
    _transform_marker: PhantomData<T>,
}

impl<'a, W, T> ValueSeqSerializer<'a, W, T> {
    pub(super) fn new(len: Option<usize>) -> Self {
        Self {
            inner: ValueSerializer {
                container: NoFieldTransforms,
                _marker: PhantomData,
            },
            values: Vec::with_capacity(len.unwrap_or(8)),
            _transform_marker: PhantomData,
        }
    }
}

impl<W: WriteKind, T> ser::SerializeSeq for ValueSeqSerializer<'_, W, T> {
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    fn serialize_element<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        let SerializedValueKind::Value(value) = self.inner.serialize(value)?;

        if W::MERGE && matches!(value, NullValue(_)) {
            return Ok(());
        }

        self.values.push(firestore::Value {
            value_type: Some(value),
        });

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::ArrayValue(
            ArrayValue {
                values: self.values,
            },
        )))
    }
}

impl<W: WriteKind, T> ser::SerializeTuple for ValueSeqSerializer<'_, W, T> {
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    #[inline]
    fn serialize_element<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<W: WriteKind, T> ser::SerializeTupleStruct for ValueSeqSerializer<'_, W, T> {
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    #[inline]
    fn serialize_field<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<W: WriteKind, T> ser::SerializeTupleVariant for ValueSeqSerializer<'_, W, T> {
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    #[inline]
    fn serialize_field<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}
