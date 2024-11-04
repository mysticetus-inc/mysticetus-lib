#![allow(dead_code)]

use std::borrow::Cow;

use protos::protobuf::value::Kind;
use protos::spanner::TypeCode;
use serde::de;

use crate::error::ConvertError;
use crate::{Field, Type, Value};

pub struct ValueDeserializer<'a> {
    pub(super) ty: Option<Cow<'a, Type>>,
    pub(super) value: Value,
}

impl ValueDeserializer<'_> {
    pub fn deserialize<'de, T>(field: &Field, value: Value) -> Result<T, ConvertError>
    where
        T: serde::Deserialize<'de>,
    {
        T::deserialize(ValueDeserializer {
            ty: Some(Cow::Borrowed(&field.ty)),
            value,
        })
    }
}

impl<'de> serde::Deserializer<'de> for ValueDeserializer<'_> {
    type Error = ConvertError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        use Kind::*;
        match self.value.0 {
            NullValue(_) => visitor.visit_unit(),
            NumberValue(num) => visitor.visit_f64(num),
            StringValue(s) => match self.ty.as_deref().map(|t| t.to_type_code()) {
                Some(TypeCode::Float64) => match s.as_str() {
                    "NaN" => visitor.visit_f64(f64::NAN),
                    "Infinity" => visitor.visit_f64(f64::INFINITY),
                    "-Infinity" => visitor.visit_f64(f64::NEG_INFINITY),
                    _ => visitor.visit_string(s),
                },
                _ => visitor.visit_string(s),
            },
            BoolValue(b) => visitor.visit_bool(b),
            StructValue(s) => visitor.visit_map(ValueMapAccess {
                fields: self.ty.as_ref().and_then(|t| t.get_struct_fields()),
                values: s.fields.into_iter(),
                next_value_and_ty: None,
            }),
            ListValue(l) => visitor.visit_seq(SeqAccess {
                elem_ty: self.ty.as_ref().and_then(|t| t.get_array_elem()),
                values: l.values.into_iter(),
            }),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.value.0 {
            Kind::NullValue(_) => visitor.visit_none(),
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
        use Kind::*;
        use serde::de::Error;
        use serde::de::value::{MapAccessDeserializer, StringDeserializer, U32Deserializer};

        match self.value.0 {
            // cant deserialize the following into enums:
            NullValue(_) => Err(ConvertError::invalid_value(
                de::Unexpected::Option,
                &"a non-null value",
            )),
            BoolValue(b) => Err(ConvertError::invalid_value(
                de::Unexpected::Bool(b),
                &"an enum compatible value",
            )),
            ListValue(_) => Err(ConvertError::invalid_value(
                de::Unexpected::Seq,
                &"an enum compatible value",
            )),
            // The remaining variants can be, but usually with some asterisks.
            // Needs to be an integer, since integers can be used to determine the enum variant (for
            // unit variants)
            NumberValue(num) if num.trunc() == num => {
                visitor.visit_enum(U32Deserializer::new(num as u32))
            }
            NumberValue(float) => Err(ConvertError::invalid_value(
                de::Unexpected::Float(float),
                &"an enum compatible value",
            )),
            StringValue(s) => match self.ty.as_deref().map(|t| t.to_type_code()) {
                // integers are encoded as strings, so parse that if we need to
                Some(TypeCode::Int64) => match s.parse() {
                    Ok(parsed) => visitor.visit_enum(U32Deserializer::new(parsed)),
                    Err(_) => Err(ConvertError::invalid_value(
                        de::Unexpected::Str(&s),
                        &"a valid integer",
                    )),
                },
                _ => visitor.visit_enum(StringDeserializer::new(s)),
            },
            StructValue(s) => visitor.visit_enum(MapAccessDeserializer::new(ValueMapAccess {
                fields: self.ty.as_ref().and_then(|t| t.get_struct_fields()),
                values: s.fields.into_iter(),
                next_value_and_ty: None,
            })),
        }
    }

    serde::forward_to_deserialize_any! {
        i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 bool char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

pub struct ValueMapAccess<'a> {
    fields: Option<Cow<'a, [Field]>>,
    values: std::collections::hash_map::IntoIter<String, protos::protobuf::Value>,
    next_value_and_ty: Option<(Value, Option<usize>)>,
}

impl<'a> ValueMapAccess<'a> {
    fn try_find_field_index(&self, key: &str) -> Option<usize> {
        self.fields
            .as_ref()?
            .iter()
            .position(|field| field.name == key)
    }
}

impl<'de> serde::de::MapAccess<'de> for ValueMapAccess<'_> {
    type Error = ConvertError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some((key, value)) => {
                self.next_value_and_ty =
                    Some((Value::from_proto(value), self.try_find_field_index(&key)));

                seed.deserialize(de::IntoDeserializer::into_deserializer(key))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let (value, index) = self.next_value_and_ty.take().unwrap();
        seed.deserialize(ValueDeserializer {
            ty: index.map(|idx| Cow::Borrowed(&self.fields.as_ref().unwrap()[idx].ty)),
            value,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

pub struct SeqAccess<'a> {
    elem_ty: Option<Cow<'a, Type>>,
    values: std::vec::IntoIter<protos::protobuf::Value>,
}

impl<'de> serde::de::SeqAccess<'de> for SeqAccess<'_> {
    type Error = ConvertError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(value) => {
                let de = ValueDeserializer {
                    ty: self.elem_ty.as_deref().map(Cow::Borrowed),
                    value: Value::from_proto(value),
                };

                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}
