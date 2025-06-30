use std::collections::HashMap;
use std::convert::Infallible;

use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::TransformType;
use protos::firestore::value::ValueType::{self, NullValue};
use protos::firestore::{self, MapValue};
use serde::{Serialize, ser};
use serde_helpers::key_capture::KeyCapture;

use crate::error::SerError;
use crate::ser::field_transform::Transform;
use crate::ser::value::SerializedValueKind;
use crate::ser::{MapSerializerKind, ValueSerializer, WriteKind};

pub(crate) struct ValueMapSerializer<'a, W, T: Transform> {
    inner: ValueSerializer<'a, W, T>,
    fields: HashMap<String, firestore::Value>,
    key: Option<String>,
}

impl<'a, W: WriteKind> MapSerializerKind<ValueSerializer<'a, W, Infallible>>
    for ValueMapSerializer<'a, W, Infallible>
{
    type Output = SerializedValueKind<Infallible>;

    fn new_with_len(len: Option<usize>, inner: ValueSerializer<'a, W, Infallible>) -> Self {
        Self {
            inner,
            fields: HashMap::with_capacity(len.unwrap_or(8)),
            key: None,
        }
    }
}

impl<'a, W: WriteKind> MapSerializerKind<ValueSerializer<'a, W, TransformType>>
    for ValueMapSerializer<'a, W, TransformType>
{
    type Output = SerializedValueKind<TransformType>;

    fn new_with_len(len: Option<usize>, inner: ValueSerializer<'a, W, TransformType>) -> Self {
        Self {
            inner,
            fields: HashMap::with_capacity(len.unwrap_or(8)),
            key: None,
        }
    }
}

impl<W: WriteKind> ser::SerializeMap for ValueMapSerializer<'_, W, Infallible> {
    type Ok = SerializedValueKind<Infallible>;
    type Error = SerError;

    fn serialize_key<S>(&mut self, key: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        key.serialize(KeyCapture(&mut self.key))
            .map_err(<SerError as serde::ser::Error>::custom)
    }

    fn serialize_value<S>(&mut self, value: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        let mut key = self
            .key
            .take()
            .expect("serialize_value called without calling serialize_key");

        let SerializedValueKind::Value(value) = self.inner.serialize(value)?;

        if W::MERGE && matches!(value, NullValue(_)) {
            key.clear();
            self.key = Some(key);
            return Ok(());
        }

        self.fields.insert(
            key,
            firestore::Value {
                value_type: Some(value),
            },
        );

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::MapValue(MapValue {
            fields: self.fields,
        })))
    }
}

impl<W: WriteKind> ser::SerializeMap for ValueMapSerializer<'_, W, TransformType> {
    type Ok = SerializedValueKind<TransformType>;
    type Error = SerError;

    fn serialize_key<S>(&mut self, key: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        key.serialize(&mut self.inner.container.path)
    }

    fn serialize_value<S>(&mut self, value: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        match self.inner.serialize(value)? {
            SerializedValueKind::Value(value) => {
                if W::MERGE && matches!(value, NullValue(_)) {
                    self.inner.container.path.pop();
                    return Ok(());
                }

                let key = self
                    .inner
                    .container
                    .path
                    .pop_take()
                    .expect("serialize_value called without calling serialize_key")
                    .into_owned();

                self.fields.insert(
                    key,
                    firestore::Value {
                        value_type: Some(value),
                    },
                );
                Ok(())
            }
            SerializedValueKind::Transform(transform) => {
                let field_path = self.inner.container.path.make_path();
                self.inner.container.path.pop();

                self.inner.container.transforms.push(FieldTransform {
                    field_path,
                    transform_type: Some(transform),
                });

                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerializedValueKind::Value(ValueType::MapValue(MapValue {
            fields: self.fields,
        })))
    }
}

impl<W: WriteKind> ser::SerializeStruct for ValueMapSerializer<'_, W, std::convert::Infallible> {
    type Ok = SerializedValueKind<std::convert::Infallible>;
    type Error = SerError;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        let SerializedValueKind::Value(value) = self.inner.serialize(value)?;

        if W::MERGE && matches!(value, NullValue(_)) {
            return Ok(());
        }

        self.fields.insert(
            key.to_owned(),
            firestore::Value {
                value_type: Some(value),
            },
        );

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}

impl<W: WriteKind> ser::SerializeStruct for ValueMapSerializer<'_, W, TransformType> {
    type Ok = SerializedValueKind<TransformType>;
    type Error = SerError;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        self.inner.container.path.push_static(key);

        match self.inner.serialize(value)? {
            SerializedValueKind::Value(value) => {
                self.inner.container.path.pop();

                if W::MERGE && matches!(value, NullValue(_)) {
                    return Ok(());
                }

                self.fields.insert(
                    key.to_owned(),
                    firestore::Value {
                        value_type: Some(value),
                    },
                );
                Ok(())
            }
            SerializedValueKind::Transform(transform) => {
                let field_path = self.inner.container.path.make_path();
                self.inner.container.path.pop();

                self.inner.container.transforms.push(FieldTransform {
                    field_path,
                    transform_type: Some(transform),
                });

                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}

impl<W: WriteKind, T: Transform> ser::SerializeStructVariant for ValueMapSerializer<'_, W, T>
where
    Self: ser::SerializeStruct<Ok = SerializedValueKind<T>, Error = SerError>,
{
    type Ok = SerializedValueKind<T>;
    type Error = SerError;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        <Self as ser::SerializeStruct>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeStruct>::end(self)
    }
}
