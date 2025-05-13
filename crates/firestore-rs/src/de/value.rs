use std::collections::{HashMap, hash_map};
use std::vec;

use protos::firestore::value::ValueType;
use protos::firestore::{self, ArrayValue, MapValue};
use protos::r#type::LatLng;
use serde::de::{self, Deserializer, IntoDeserializer};
use serde::forward_to_deserialize_any;
use timestamp::Timestamp;

use super::enum_de::{RootEnum, StringEnum};
use crate::ConvertError;

pub struct ValueDeserializer {
    value: firestore::Value,
}

impl From<firestore::Value> for ValueDeserializer {
    fn from(value: firestore::Value) -> Self {
        Self { value }
    }
}

impl<'de> Deserializer<'de> for ValueDeserializer {
    type Error = ConvertError;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value.value_type {
            None | Some(ValueType::NullValue(_)) => visitor.visit_unit(),
            Some(ValueType::BooleanValue(boolean)) => visitor.visit_bool(boolean),
            Some(ValueType::IntegerValue(integer)) => visitor.visit_i64(integer),
            Some(ValueType::DoubleValue(float)) => visitor.visit_f64(float),
            Some(ValueType::TimestampValue(ts)) => {
                let epoch = Timestamp::from(ts).as_seconds_f64();
                visitor.visit_f64(epoch)
            }
            Some(ValueType::StringValue(string)) | Some(ValueType::ReferenceValue(string)) => {
                visitor.visit_string(string)
            }
            Some(ValueType::BytesValue(bytes)) => visitor.visit_byte_buf(bytes.into()),
            Some(ValueType::GeoPointValue(lat_lng)) => {
                visitor.visit_seq(LonLatAccess::from(lat_lng))
            }
            Some(ValueType::ArrayValue(ArrayValue { values })) => {
                visitor.visit_seq(ValueSeqAccess::new(values))
            }
            Some(ValueType::MapValue(MapValue { fields })) => {
                visitor.visit_map(ValueMapAccess::new(fields))
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value.value_type {
            None | Some(ValueType::NullValue(_)) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value.value_type {
            Some(ValueType::StringValue(string)) => visitor.visit_enum(StringEnum::new(string)),
            Some(ValueType::MapValue(MapValue { fields })) => {
                visitor.visit_enum(RootEnum::new(fields))
            }
            _ => Err(ConvertError::de(format!(
                "found {:?}, which cannot be serialized into an enum",
                self.value.value_type,
            ))),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match (name, self.value.value_type) {
            (crate::timestamp::NEWTYPE_MARKER, Some(ValueType::TimestampValue(ts))) => {
                let nanos = Timestamp::from(ts).as_nanos();
                visitor.visit_i128(nanos)
            }
            (_, value_type) => Self::from(firestore::Value { value_type }).deserialize_any(visitor),
        }
    }
}

struct LonLatAccess {
    lon: Option<f64>,
    lat: Option<f64>,
}

impl From<LatLng> for LonLatAccess {
    fn from(lat_lng: LatLng) -> Self {
        Self {
            lon: Some(lat_lng.longitude),
            lat: Some(lat_lng.latitude),
        }
    }
}

impl<'de> de::SeqAccess<'de> for LonLatAccess {
    type Error = ConvertError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.lon.take().or_else(|| self.lat.take()) {
            Some(coord) => seed.deserialize(coord.into_deserializer()).map(Some),
            _ => Ok(None),
        }
    }
}

pub(super) struct ValueSeqAccess {
    iter: vec::IntoIter<firestore::Value>,
}

impl ValueSeqAccess {
    pub(super) fn new(values: Vec<firestore::Value>) -> Self {
        Self {
            iter: values.into_iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for ValueSeqAccess {
    type Error = ConvertError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = match self.iter.next() {
            Some(value) => value,
            _ => return Ok(None),
        };

        seed.deserialize(ValueDeserializer::from(value)).map(Some)
    }
}

pub(super) struct ValueMapAccess {
    iter: hash_map::IntoIter<String, firestore::Value>,
    curr_value: Option<firestore::Value>,
}

impl ValueMapAccess {
    pub(super) fn new(fields: HashMap<String, firestore::Value>) -> Self {
        Self {
            iter: fields.into_iter(),
            curr_value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for ValueMapAccess {
    type Error = ConvertError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let (key, value) = match self.iter.next() {
            Some(kvp) => kvp,
            _ => return Ok(None),
        };

        self.curr_value = Some(value);

        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .curr_value
            .take()
            .expect("no value to match previous key");

        seed.deserialize(ValueDeserializer::from(value))
    }
}
