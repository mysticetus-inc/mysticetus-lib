use std::borrow::Cow;

use crate::json::{JsonValue, Number, Primitive};

pub(crate) struct JsonSerializer;

impl JsonSerializer {
    pub fn serialize_or_null(value: impl serde::Serialize) -> JsonValue {
        value.serialize(JsonSerializer).unwrap_or(JsonValue::NULL)
    }
    pub fn try_serialize(value: impl serde::Serialize) -> Option<JsonValue> {
        value.serialize(JsonSerializer).ok()
    }
    pub fn serialize(value: impl serde::Serialize) -> JsonValue {
        match value.serialize(JsonSerializer) {
            Ok(serialized) => serialized,
            Err(JsonError) => JsonError::make_error_value(),
        }
    }
}

/// An unexpected error (that we'll ignore) encountered when serializing to json.
///
/// Only ever constructed via the [serde::ser::Error] impl, so
/// unless a user implemented serde::Serialize impl creates one,
/// we should never see any errors.
#[derive(Debug, thiserror::Error)]
#[error("{}", Self::ERR_MESSAGE)]
pub(crate) struct JsonError;

impl JsonError {
    const ERR_MESSAGE: &str = "unexpected json serialization error";

    pub const MARKER_KEY: &str = "__json_error__";

    fn make_error_value() -> JsonValue {
        let mut map = fxhash::FxHashMap::with_capacity_and_hasher(1, fxhash::FxBuildHasher::new());
        map.insert(Cow::Borrowed(Self::MARKER_KEY), JsonValue::TRUE);
        JsonValue::Map(map)
    }
}

impl serde::ser::Error for JsonError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        JsonError
    }
}

impl serde::Serializer for JsonSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;

    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructSerializer;

    #[inline]
    fn serialize_bool(self, b: bool) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Bool(b)))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<JsonValue, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<JsonValue, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<JsonValue, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Number(Number::Int(value))))
    }

    #[inline]
    fn serialize_i128(self, value: i128) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Number(Number::BigInt(
            value,
        ))))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<JsonValue, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<JsonValue, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<JsonValue, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Number(Number::Uint(value))))
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Number(Number::BigUint(
            value,
        ))))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<JsonValue, JsonError> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Number(Number::Float(
            value,
        ))))
    }

    #[inline]
    fn serialize_char(self, ch: char) -> Result<JsonValue, JsonError> {
        self.serialize_str(ch.encode_utf8(&mut [0u8; 4]))
    }

    #[inline]
    fn serialize_str(self, s: &str) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Str(Box::from(s))))
    }

    #[inline]
    fn serialize_bytes(self, bytes: &[u8]) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Str(
            String::from_utf8_lossy(bytes).into_owned().into_boxed_str(),
        )))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<JsonValue, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_none(self) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Null))
    }

    #[inline]
    fn serialize_unit(self) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Primitive(Primitive::Null))
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<JsonValue, JsonError> {
        self.serialize_str(name)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
    ) -> Result<JsonValue, JsonError> {
        self.serialize_str(name)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<JsonValue, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T,
    ) -> Result<JsonValue, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, JsonError> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len.unwrap_or(8)),
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer, JsonError> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len),
        })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, JsonError> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len),
        })
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, JsonError> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len),
        })
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<MapSerializer, JsonError> {
        Ok(MapSerializer {
            map: fxhash::FxHashMap::with_capacity_and_hasher(
                len.unwrap_or(8),
                fxhash::FxBuildHasher::default(),
            ),
            next_key: NextKey::Empty,
        })
    }

    #[inline]
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<StructSerializer, JsonError> {
        Ok(StructSerializer {
            map: fxhash::FxHashMap::with_capacity_and_hasher(len, fxhash::FxBuildHasher::default()),
        })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        len: usize,
    ) -> Result<StructSerializer, JsonError> {
        Ok(StructSerializer {
            map: fxhash::FxHashMap::with_capacity_and_hasher(len, fxhash::FxBuildHasher::default()),
        })
    }
}

pub(crate) struct SeqSerializer {
    values: Vec<JsonValue>,
}

pub(crate) struct MapSerializer {
    map: fxhash::FxHashMap<Cow<'static, str>, JsonValue>,
    next_key: NextKey,
}

pub(crate) enum NextKey {
    Empty,
    Skip,
    Next(Cow<'static, str>),
}

pub(crate) struct StructSerializer {
    map: fxhash::FxHashMap<Cow<'static, str>, JsonValue>,
}

impl serde::ser::SerializeSeq for SeqSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        self.values.push(JsonSerializer::serialize(value));
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Array(self.values))
    }
}

impl serde::ser::SerializeTuple for SeqSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SeqSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SeqSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeMap for MapSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        self.next_key = match key.serialize(KeySerializer) {
            Ok(Key::Str(s)) => NextKey::Next(s),
            Ok(Key::Bool(true)) => NextKey::Next(Cow::Borrowed("true")),
            Ok(Key::Bool(false)) => NextKey::Next(Cow::Borrowed("false")),
            Ok(Key::Number(num)) => num.visit_str(|s| NextKey::Next(Cow::Owned(s.to_owned()))),
            Ok(Key::Null) => NextKey::Next(Cow::Borrowed("null")),
            Ok(Key::Map | Key::Seq) => NextKey::Skip,
            Err(JsonError) => NextKey::Skip,
        };

        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        match std::mem::replace(&mut self.next_key, NextKey::Empty) {
            NextKey::Empty => panic!("serialize_value called without calling serialize_key"),
            NextKey::Skip => Ok(()),
            NextKey::Next(key) => {
                let value = JsonSerializer::serialize(value);
                self.map.insert(key, value);
                Ok(())
            }
        }
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Map(self.map))
    }
}

impl serde::ser::SerializeStruct for StructSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        let value = JsonSerializer::serialize(value);
        self.map.insert(Cow::Borrowed(key), value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Map(self.map))
    }
}

impl serde::ser::SerializeStructVariant for StructSerializer {
    type Ok = JsonValue;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<JsonValue, JsonError> {
        serde::ser::SerializeStruct::end(self)
    }
}

struct KeySerializer;

enum Key {
    Str(Cow<'static, str>),
    Number(Number),
    Bool(bool),
    Null,
    Map,
    Seq,
}

struct IntoKey(Key);

impl serde::Serializer for KeySerializer {
    type Ok = Key;
    type Error = JsonError;

    type SerializeSeq = IntoKey;
    type SerializeTuple = IntoKey;
    type SerializeTupleStruct = IntoKey;
    type SerializeTupleVariant = IntoKey;

    type SerializeMap = IntoKey;
    type SerializeStruct = IntoKey;
    type SerializeStructVariant = IntoKey;

    #[inline]
    fn serialize_bool(self, b: bool) -> Result<Key, JsonError> {
        Ok(Key::Bool(b))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Key, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Key, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Key, JsonError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Key, JsonError> {
        Ok(Key::Number(Number::Int(value)))
    }

    #[inline]
    fn serialize_i128(self, value: i128) -> Result<Key, JsonError> {
        Ok(Key::Number(Number::BigInt(value)))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Key, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Key, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Key, JsonError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Key, JsonError> {
        Ok(Key::Number(Number::Uint(value)))
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<Key, JsonError> {
        Ok(Key::Number(Number::BigUint(value)))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Key, JsonError> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Key, JsonError> {
        Ok(Key::Number(Number::Float(value)))
    }

    #[inline]
    fn serialize_char(self, ch: char) -> Result<Key, JsonError> {
        self.serialize_str(ch.encode_utf8(&mut [0u8; 4]))
    }

    #[inline]
    fn serialize_str(self, s: &str) -> Result<Key, JsonError> {
        Ok(Key::Str(Cow::Owned(s.to_owned())))
    }

    #[inline]
    fn serialize_bytes(self, bytes: &[u8]) -> Result<Key, JsonError> {
        Ok(Key::Str(Cow::Owned(
            String::from_utf8_lossy(bytes).into_owned(),
        )))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Key, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_none(self) -> Result<Key, JsonError> {
        Ok(Key::Null)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Key, JsonError> {
        Ok(Key::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Key, JsonError> {
        Ok(Key::Str(Cow::Borrowed(name)))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
    ) -> Result<Key, JsonError> {
        Ok(Key::Str(Cow::Borrowed(name)))
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Key, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T,
    ) -> Result<Key, JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, _: Option<usize>) -> Result<IntoKey, JsonError> {
        Ok(IntoKey(Key::Seq))
    }

    #[inline]
    fn serialize_tuple(self, _: usize) -> Result<IntoKey, JsonError> {
        Ok(IntoKey(Key::Seq))
    }

    #[inline]
    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<IntoKey, JsonError> {
        if len == 0 {
            Ok(IntoKey(Key::Str(Cow::Borrowed(name))))
        } else {
            Ok(IntoKey(Key::Seq))
        }
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
        len: usize,
    ) -> Result<IntoKey, JsonError> {
        self.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_map(self, _: Option<usize>) -> Result<IntoKey, JsonError> {
        Ok(IntoKey(Key::Map))
    }

    #[inline]
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<IntoKey, JsonError> {
        Ok(IntoKey(Key::Map))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<IntoKey, JsonError> {
        Ok(IntoKey(Key::Map))
    }
}

impl serde::ser::SerializeSeq for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_element<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeTuple for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_element<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeTupleStruct for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeTupleVariant for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeMap for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_key<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeStruct for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}

impl serde::ser::SerializeStructVariant for IntoKey {
    type Ok = Key;
    type Error = JsonError;

    #[inline]
    fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), JsonError>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Key, JsonError> {
        Ok(self.0)
    }
}
