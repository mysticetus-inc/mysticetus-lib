use std::convert::Infallible;
use std::fmt;
use std::sync::Arc;

use bstr::BStr;
use bytes::Buf;
use serde::de::Error as SerdeError;

use crate::Error;
use crate::reader::{RawRow, RawRowIter};

#[derive(Clone)]
pub struct Row {
    headers: Arc<RawRow>,
    row: RawRow,
}

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugRowData<'a>(&'a Row);

        impl fmt::Debug for DebugRowData<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut map = f.debug_map();

                for (header, field) in self.0 {
                    map.entry(&header, &field);
                }

                map.finish()
            }
        }

        f.debug_struct("Row")
            .field("line", &self.row.line())
            .field("fields", &DebugRowData(self))
            .finish()
    }
}

impl Row {
    pub(crate) fn new(headers: Arc<RawRow>, row: RawRow) -> Self {
        Self { headers, row }
    }
}

impl<'a> IntoIterator for &'a Row {
    type IntoIter = std::iter::Zip<RawRowIter<'a>, RawRowIter<'a>>;
    type Item = (&'a BStr, &'a BStr);

    fn into_iter(self) -> Self::IntoIter {
        (&self.headers).into_iter().zip(&self.row)
    }
}

impl<'de> serde::Deserializer<'de> for &'de Row {
    type Error = Error<Infallible>;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(RowMapAccess {
            iter: self.into_iter(),
            next_value: None,
        })
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(RowSeqAccess {
            values: (&self.row).into_iter(),
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct struct enum identifier ignored_any
    }
}

struct RowMapAccess<'de> {
    next_value: Option<&'de [u8]>,
    iter: <&'de Row as IntoIterator>::IntoIter,
}

impl<'de> serde::de::MapAccess<'de> for RowMapAccess<'de> {
    type Error = Error<Infallible>;

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len() + self.next_value.is_some() as usize)
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((field, value)) => {
                self.next_value = Some(value);

                seed.deserialize(BytesDeserializer { bytes: field })
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let bytes = self
            .next_value
            .take()
            .expect("next_value_seed called without calling next_key_seed");

        seed.deserialize(BytesDeserializer { bytes })
    }
}

struct RowSeqAccess<'de> {
    values: RawRowIter<'de>,
}

impl<'de> serde::de::SeqAccess<'de> for RowSeqAccess<'de> {
    type Error = Error<Infallible>;

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.values
            .next()
            .map(|bytes| seed.deserialize(BytesDeserializer { bytes }))
            .transpose()
    }
}

struct BytesDeserializer<'de> {
    bytes: &'de [u8],
}

macro_rules! impl_int_fns {
    ($(
        $fn_name:ident($int_ty:ty, $visit_fn:ident)
    ),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>
            {
                let int = <$int_ty>::from_ascii(self.bytes).map_err(Error::custom)?;
                visitor.$visit_fn(int)
            }
        )*
    };
}

impl<'de> serde::de::Deserializer<'de> for BytesDeserializer<'de> {
    type Error = Error<Infallible>;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // assume that if we arent told what to deserialize into, we should try a string, since
        // other formats can often parse themselves from one
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.bytes.is_empty() || self.bytes.eq_ignore_ascii_case(b"null") {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = std::str::from_utf8(self.bytes).map_err(|err| {
            crate::Error::Serde(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(self.bytes),
                &err.to_string().as_str(),
            ))
        })?;

        visitor.visit_borrowed_str(s)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.bytes {
            b"0" | b"false" | b"False" | b"FALSE" => visitor.visit_bool(false),
            b"1" | b"true" | b"True" | b"TRUE" => visitor.visit_bool(true),
            _ => Err(Error::invalid_value(
                serde::de::Unexpected::Bytes(&self.bytes),
                &"a known represenation of a boolean",
            )),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.bytes {
            b"0" => return visitor.visit_f64(0.0),
            b"inf" | b"+inf" | b"Inf" | b"+Inf" => return visitor.visit_f64(f64::INFINITY),
            b"-inf" | b"-Inf" => return visitor.visit_f64(f64::NEG_INFINITY),
            b"NaN" | b"nan" | b"NAN" => return visitor.visit_f64(f64::NAN),
            _ => (),
        }

        let s = std::str::from_utf8(self.bytes).map_err(Error::custom)?;
        let float = s.parse().map_err(Error::custom)?;
        visitor.visit_f64(float)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let raw_char = match self.bytes.len() {
            // we might be a single char if we're between 1 and 4 bytes
            1 => self.bytes.get_u8() as u32,
            2 => self.bytes.get_u16() as u32,
            3 => ((self.bytes.get_u16() as u32) << 8) | (self.bytes.get_u8() as u32),
            4 => self.bytes.get_u32(),
            _ => {
                return Err(Error::invalid_value(
                    serde::de::Unexpected::Bytes(&self.bytes),
                    &"expected a single char",
                ));
            }
        };

        let char = char::from_u32(raw_char).ok_or_else(|| {
            Error::invalid_value(
                serde::de::Unexpected::Unsigned(raw_char as u64),
                &"invalid unicode char",
            )
        })?;

        visitor.visit_char(char)
    }

    impl_int_fns! {
        deserialize_u8(u8, visit_u8),
        deserialize_u16(u16, visit_u16),
        deserialize_u32(u32, visit_u32),
        deserialize_u64(u64, visit_u64),
        deserialize_u128(u128, visit_u128),
        deserialize_i8(i8, visit_i8),
        deserialize_i16(i16, visit_i16),
        deserialize_i32(i32, visit_i32),
        deserialize_i64(i64, visit_i64),
        deserialize_i128(i128, visit_i128),
    }

    serde::forward_to_deserialize_any! {
        identifier map seq unit_struct tuple tuple_struct
        struct enum ignored_any
    }
}
