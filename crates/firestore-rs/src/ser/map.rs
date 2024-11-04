//! A serializer to create maps of [`firestore::Value`] data from [`Serialize`]-able types.

use std::collections::HashMap;
use std::marker::PhantomData;

use protos::firestore;
use serde::ser::{Serialize, SerializeMap, SerializeStruct, SerializeStructVariant};
use serde_helpers::key_capture::KeyCapture;

use super::NullStrategy;
use crate::ConvertError;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MapSerializer<N> {
    curr_key: Option<String>,
    fields: HashMap<String, firestore::Value>,
    _marker: PhantomData<N>,
}

impl<N> MapSerializer<N> {
    pub(super) fn new(len: Option<usize>) -> Self {
        let fields = len.map(HashMap::with_capacity).unwrap_or_default();

        Self {
            curr_key: None,
            fields,
            _marker: PhantomData,
        }
    }
}

impl<N> SerializeMap for MapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = HashMap<String, firestore::Value>;
    type Error = ConvertError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(KeyCapture(&mut self.curr_key))
            .map_err(ConvertError::ser)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let key = std::mem::take(&mut self.curr_key)
            .ok_or_else(|| ConvertError::ser("Missing key, cannot serialize value"))?;

        match value.serialize(super::ValueSerializer::<N>::NEW)? {
            Some(val) => {
                self.fields.insert(key, val);
            }
            None => N::handle_null(|val| self.fields.insert(key, val)),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.curr_key {
            Some(leftover_key) => Err(ConvertError::ser(format!(
                "'{}' key leftover on map serialize end",
                leftover_key
            ))),
            None => Ok(self.fields),
        }
    }
}

impl<N> SerializeStruct for MapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = HashMap<String, firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match value.serialize(super::ValueSerializer::<N>::NEW)? {
            Some(val) => {
                self.fields.insert(key.to_owned(), val);
            }
            None => N::handle_null(|val| self.fields.insert(key.to_owned(), val)),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.fields)
    }
}

impl<N> SerializeStructVariant for MapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = HashMap<String, firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}
