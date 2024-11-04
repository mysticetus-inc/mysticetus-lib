//! Deserializer for firestore documents.
use std::collections::HashMap;

use protos::firestore;
use serde::de::{self, Deserializer};
use serde::forward_to_deserialize_any;

use crate::ConvertError;

mod enum_de;
use enum_de::RootEnum;

mod value;
pub(super) use value::ValueDeserializer;
use value::{ValueMapAccess, ValueSeqAccess};

/// Deserializes the fields of a [`crate::RawDoc`]. Allows for deferred deserialization by
/// retrieving raw documents.
pub fn deserialize_doc_fields<'de, O>(
    fields: HashMap<String, firestore::Value>,
) -> Result<O, ConvertError>
where
    O: de::Deserialize<'de>,
{
    O::deserialize(path_aware_serde::Deserializer::new(DocDeserializer {
        fields,
    }))
    .map_err(ConvertError::from_path_aware)
}

pub(super) struct DocDeserializer {
    fields: HashMap<String, firestore::Value>,
}

impl From<HashMap<String, firestore::Value>> for DocDeserializer {
    fn from(fields: HashMap<String, firestore::Value>) -> Self {
        Self { fields }
    }
}

impl<'de> Deserializer<'de> for DocDeserializer {
    type Error = ConvertError;

    // Since the resulting type is a struct/enum (i.e a non-primitive type), we can use
    // 'deserialize_any' to return an incorrect type error, and forward all primitives there.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(ValueMapAccess::new(self.fields))
        // Err(ConvertError::de("cannot deserialize into a non-map/struct type"))
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
        visitor.visit_enum(RootEnum::new(self.fields))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(ValueMapAccess::new(self.fields))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}
