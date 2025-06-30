use std::collections::HashMap;
use std::marker::PhantomData;

use protos::firestore;
use protos::firestore::value::ValueType::NullValue;
use serde::ser::Error;
use serde_helpers::key_capture::KeyCapture;

use crate::DocFields;
use crate::error::SerError;
use crate::ser::{MapSerializerKind, WriteKind};

pub struct Update<W: WriteKind> {
    key: Option<String>,
    fields: HashMap<String, firestore::Value>,
    _marker: PhantomData<fn(W)>,
}

impl<W: WriteKind> MapSerializerKind for Update<W> {
    type Output = DocFields;

    fn new_with_len(len: Option<usize>, _: ()) -> Self {
        Self {
            key: None,
            fields: HashMap::with_capacity(len.unwrap_or(8)),
            _marker: PhantomData,
        }
    }
}

impl<W: WriteKind> serde::ser::SerializeMap for Update<W> {
    type Ok = DocFields;
    type Error = SerError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(KeyCapture(&mut self.key))
            .map_err(SerError::custom)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut key = self
            .key
            .take()
            .expect("serialize_value called without calling serialize_key");

        let value = crate::ser::serialize_value::<W>(value)?;

        if W::MERGE && matches!(value, NullValue(_)) {
            // re-use the buf if we're skipping this field
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
        Ok(DocFields {
            fields: self.fields,
            field_mask: None,
        })
    }
}

impl<W: WriteKind> serde::ser::SerializeStruct for Update<W> {
    type Ok = DocFields;
    type Error = SerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let value = crate::ser::serialize_value::<W>(value)?;

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

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<W: WriteKind> serde::ser::SerializeStructVariant for Update<W> {
    type Ok = DocFields;
    type Error = SerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}
