use std::collections::HashMap;
use std::collections::hash_map::IntoIter;

use protos::firestore::value::ValueType;
use protos::firestore::{self, ArrayValue, MapValue};
use serde::de::value::StringDeserializer;
use serde::de::{self, EnumAccess, IntoDeserializer, VariantAccess};

use super::{ValueDeserializer, ValueMapAccess, ValueSeqAccess};
use crate::ConvertError;

pub(super) struct MapEnum {
    iter: IntoIter<String, firestore::Value>,
    value: Option<firestore::Value>,
}

impl<'de> VariantAccess<'de> for MapEnum {
    type Error = ConvertError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value.and_then(|v| v.value_type) {
            // unit variants should be named, null, or non existant more than likely.
            None | Some(ValueType::NullValue(_)) | Some(ValueType::StringValue(_)) => Ok(()),
            value => Err(ConvertError::de(format!(
                "'{:?}' cannot be deserialized into a unit variant",
                value
            ))),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = self
            .value
            .ok_or_else(|| ConvertError::de("newtype variant has no value"))?;
        seed.deserialize(ValueDeserializer::from(value))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let array = match self.value.and_then(|v| v.value_type) {
            Some(ValueType::ArrayValue(ArrayValue { values })) => values,
            other => {
                return Err(ConvertError::de(format!(
                    "expected a sequence of values, not '{:?}'",
                    other
                )));
            }
        };

        if array.len() != len {
            return Err(ConvertError::de(format!(
                "mismatched number of values, expected {}, but found {}",
                len,
                array.len()
            )));
        }

        visitor.visit_seq(ValueSeqAccess::new(array))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let map = match self.value.and_then(|v| v.value_type) {
            Some(ValueType::MapValue(MapValue { fields })) => fields,
            other => {
                return Err(ConvertError::de(format!(
                    "expected a map of values, instead found '{:?}'",
                    other
                )));
            }
        };

        visitor.visit_map(ValueMapAccess::new(map))
    }
}

pub(super) struct RootEnum {
    inner: MapEnum,
}

impl RootEnum {
    pub(super) fn new(fields: HashMap<String, firestore::Value>) -> Self {
        Self {
            inner: MapEnum {
                iter: fields.into_iter(),
                value: None,
            },
        }
    }
}

impl<'de> VariantAccess<'de> for RootEnum {
    type Error = ConvertError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(ConvertError::de(
            "root enum cannot be deserialized into a unit variant",
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.inner.newtype_variant_seed(seed)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.tuple_variant(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.struct_variant(fields, visitor)
    }
}

impl<'de> EnumAccess<'de> for RootEnum {
    type Error = ConvertError;
    type Variant = MapEnum;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let mut enum_vis = self.inner;

        let (key, value) = enum_vis
            .iter
            .next()
            .ok_or_else(|| ConvertError::de("enum map cannot be empty"))?;

        enum_vis.value = Some(value);

        let deserialized =
            seed.deserialize::<StringDeserializer<ConvertError>>(key.into_deserializer())?;

        Ok((deserialized, enum_vis))
    }
}

pub struct StringEnum {
    variant: Option<String>,
}

impl StringEnum {
    pub fn new(string: String) -> Self {
        Self {
            variant: Some(string),
        }
    }
}

impl<'de> EnumAccess<'de> for StringEnum {
    type Error = ConvertError;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self
            .variant
            .take()
            .ok_or_else(|| ConvertError::de("no string remaining to deserialize"))?;

        let deserialized =
            seed.deserialize::<StringDeserializer<ConvertError>>(variant.into_deserializer())?;

        Ok((deserialized, self))
    }
}

impl<'de> VariantAccess<'de> for StringEnum {
    type Error = ConvertError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.variant {
            Some(string) => seed.deserialize(string.into_deserializer()),
            _ => seed.deserialize(().into_deserializer()),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(ConvertError::de("cannot deserialize tuple variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(ConvertError::de("cannot deserialize struct variant"))
    }
}
