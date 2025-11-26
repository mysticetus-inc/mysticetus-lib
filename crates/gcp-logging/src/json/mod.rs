use std::borrow::Cow;

pub(crate) mod serializer;
pub(crate) use serializer::{JsonError, JsonSerializer};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Primitive(Primitive),
    Array(Vec<JsonValue>),
    Map(fxhash::FxHashMap<Cow<'static, str>, JsonValue>),
}

impl JsonValue {
    #[allow(dead_code)]
    pub const ZERO: Self = Self::Primitive(Primitive::Number(Number::Int(0)));

    pub const NULL: Self = Self::Primitive(Primitive::Null);
    pub const TRUE: Self = Self::Primitive(Primitive::Bool(true));
    #[allow(dead_code)]
    pub const FALSE: Self = Self::Primitive(Primitive::Bool(false));

    #[allow(dead_code)]
    pub fn is_error_marker(&self) -> bool {
        match self {
            Self::Map(map) => map
                .get(JsonError::MARKER_KEY)
                .is_some_and(|value| value == &JsonValue::TRUE),
            _ => false,
        }
    }
}

impl<T> From<Option<T>> for JsonValue
where
    T: Into<JsonValue>,
{
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::NULL,
        }
    }
}

impl From<Primitive> for JsonValue {
    #[inline]
    fn from(value: Primitive) -> Self {
        Self::Primitive(value)
    }
}

impl From<Number> for JsonValue {
    #[inline]
    fn from(value: Number) -> Self {
        Self::Primitive(Primitive::Number(value))
    }
}

impl From<bool> for JsonValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Primitive(Primitive::Bool(value))
    }
}

impl From<Box<str>> for JsonValue {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::Primitive(Primitive::Str(value))
    }
}

/// A non-nested json value.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Null,
    Bool(bool),
    Number(Number),
    Str(Box<str>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i64),
    BigInt(i128),
    BigUint(u128),
    Uint(u64),
    Float(f64),
}

impl Number {
    pub fn visit_str<O>(&self, visitor: impl FnOnce(&str) -> O) -> O {
        match *self {
            Self::Int(int) => visitor(itoa::Buffer::new().format(int)),
            Self::Uint(uint) => visitor(itoa::Buffer::new().format(uint)),
            Self::BigInt(int) => visitor(itoa::Buffer::new().format(int)),
            Self::BigUint(uint) => visitor(itoa::Buffer::new().format(uint)),
            Self::Float(float) => visitor(ryu::Buffer::new().format(float)),
        }
    }
}

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::Int(int) => serializer.serialize_i64(int),
            Self::Uint(uint) => serializer.serialize_u64(uint),
            Self::Float(f) => crate::utils::JsonFloat(f).serialize(serializer),
            Self::BigInt(int) => serializer.serialize_i128(int),
            Self::BigUint(uint) => serializer.serialize_u128(uint),
        }
    }
}

impl serde::Serialize for Primitive {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::Null => serializer.serialize_unit(),
            Self::Number(num) => num.serialize(serializer),
            Self::Str(ref s) => serializer.serialize_str(s),
            Self::Bool(b) => serializer.serialize_bool(b),
        }
    }
}

impl serde::Serialize for JsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Primitive(p) => p.serialize(serializer),
            Self::Array(arr) => serializer.collect_seq(arr.iter()),
            Self::Map(map) => serializer.collect_map(map.iter()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for JsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match deserializer.deserialize_any(Visitor)? {
            Some(value) => Ok(value),
            None => Ok(JsonValue::NULL),
        }
    }
}

struct Visitor;

impl<'de> serde::de::DeserializeSeed<'de> for Visitor {
    type Value = Option<JsonValue>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Option<JsonValue>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an valid json value")
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Number(Number::Int(
            v,
        )))))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Number(
            Number::BigInt(v),
        ))))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Number(Number::Uint(
            v,
        )))))
    }

    #[inline]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Number(
            Number::BigUint(v),
        ))))
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Number(
            Number::Float(v),
        ))))
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Bool(v))))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(JsonValue::Primitive(Primitive::Str(Box::from(v)))))
        }
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(JsonValue::Primitive(Primitive::Str(
            v.into_boxed_str(),
        ))))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(JsonValue::Primitive(Primitive::Str(
                match std::str::from_utf8(v) {
                    Ok(s) => Box::from(s),
                    Err(_) => hex::encode(v).into_boxed_str(),
                },
            ))))
        }
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(JsonValue::Primitive(Primitive::Str(
                match String::from_utf8(v) {
                    Ok(s) => s.into_boxed_str(),
                    Err(err) => hex::encode(err.as_bytes()).into_boxed_str(),
                },
            ))))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let Some((first, remaining_hint)) = deserialize_first_element(&mut seq)? else {
            return Ok(None);
        };

        let mut vec = Vec::with_capacity(remaining_hint.unwrap_or(8));
        vec.push(first);

        while let Some(maybe_value) = seq.next_element_seed(Visitor)? {
            if let Some(value) = maybe_value {
                vec.push(value);
            }
        }

        Ok(Some(JsonValue::Array(vec)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut buf = None;

        let Some((key, value)) = deserialize_kvp(&mut map, &mut buf)? else {
            return Ok(None);
        };

        let mut dst = fxhash::FxHashMap::with_capacity_and_hasher(
            map.size_hint().unwrap_or(8),
            fxhash::FxBuildHasher::default(),
        );

        dst.insert(key, value);

        while let Some((key, value)) = deserialize_kvp(&mut map, &mut buf)? {
            dst.insert(key, value);
        }

        Ok(Some(JsonValue::Map(dst)))
    }
}

fn deserialize_first_element<'de, S>(
    seq: &mut S,
) -> Result<Option<(JsonValue, Option<usize>)>, S::Error>
where
    S: serde::de::SeqAccess<'de>,
{
    let mut size_hint = seq.size_hint();

    loop {
        match seq.next_element_seed(Visitor)? {
            Some(maybe_value) => {
                if let Some(ref mut hint) = size_hint {
                    *hint = hint.saturating_sub(1);
                }

                if let Some(value) = maybe_value {
                    return Ok(Some((value, size_hint)));
                }
            }
            None => return Ok(None),
        }
    }
}

fn deserialize_kvp<'de, M>(
    map: &mut M,
    buf: &mut Option<String>,
) -> Result<Option<(Cow<'static, str>, JsonValue)>, M::Error>
where
    M: serde::de::MapAccess<'de>,
{
    macro_rules! fmt_int_key {
        ($v:expr) => {{ itoa::Buffer::new().format($v).to_owned() }};
    }

    loop {
        let Some(key) = map.next_key_seed(KeyCapture { buf })? else {
            return Ok(None);
        };

        if let Some(value) = map.next_value_seed(Visitor)? {
            let key = match key {
                StrOrNumber::Buffered => buf.take().expect("we said we buffered a string"),
                StrOrNumber::Str(key) => key.into_owned(),
                StrOrNumber::Number(Number::Int(i)) => fmt_int_key!(i),
                StrOrNumber::Number(Number::Uint(u)) => fmt_int_key!(u),
                StrOrNumber::Number(Number::BigInt(i)) => fmt_int_key!(i),
                StrOrNumber::Number(Number::BigUint(u)) => fmt_int_key!(u),
                StrOrNumber::Number(Number::Float(_)) => unreachable!("we don't capture floats"),
            };

            return Ok(Some((Cow::Owned(key), value)));
        }
    }
}

struct KeyCapture<'a> {
    buf: &'a mut Option<String>,
}

enum StrOrNumber<'de> {
    Buffered,
    Str(Cow<'de, str>),
    Number(Number),
}

impl<'de> serde::de::DeserializeSeed<'de> for KeyCapture<'_> {
    type Value = StrOrNumber<'de>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(self)
    }
}

impl<'de> serde::de::Visitor<'de> for KeyCapture<'_> {
    type Value = StrOrNumber<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid json object key (i.e a string)")
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrOrNumber::Number(Number::Int(v)))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrOrNumber::Number(Number::BigInt(v)))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrOrNumber::Number(Number::Uint(v)))
    }

    #[inline]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrOrNumber::Number(Number::BigUint(v)))
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrOrNumber::Str(Cow::Borrowed(v)))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            return Ok(StrOrNumber::Str(Cow::Borrowed("")));
        }

        match self.buf {
            Some(buf) => {
                buf.clear();
                buf.push_str(v);
            }
            None => _ = self.buf.insert(v.to_owned()),
        }

        Ok(StrOrNumber::Buffered)
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            return Ok(StrOrNumber::Str(Cow::Borrowed("")));
        }

        match self.buf {
            Some(_) => Ok(StrOrNumber::Str(Cow::Owned(v))),
            None => {
                *self.buf = Some(v);
                Ok(StrOrNumber::Buffered)
            }
        }
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            return Ok(StrOrNumber::Str(Cow::Borrowed("")));
        }

        match std::str::from_utf8(v) {
            Ok(s) => Ok(StrOrNumber::Str(Cow::Borrowed(s))),
            Err(_) => Ok(encode_hex_into(v, self.buf)),
        }
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            return Ok(StrOrNumber::Str(Cow::Borrowed("")));
        }

        match std::str::from_utf8(v) {
            Ok(s) => {
                match self.buf {
                    Some(buf) => {
                        buf.clear();
                        buf.push_str(s);
                    }
                    None => _ = self.buf.insert(s.to_owned()),
                }
                Ok(StrOrNumber::Buffered)
            }
            Err(_) => Ok(encode_hex_into(v, self.buf)),
        }
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            if self.buf.is_none() && 0 < v.capacity() {
                *self.buf = Some(
                    String::from_utf8(v).expect("the vec is empty, so its a valid empty string"),
                );
            }
            return Ok(StrOrNumber::Str(Cow::Borrowed("")));
        }

        match String::from_utf8(v) {
            Ok(s) => Ok(StrOrNumber::Str(Cow::Owned(s))),
            Err(err) => {
                let mut v = err.into_bytes();
                let encoded = hex::encode(&v);

                if self.buf.is_none() {
                    v.clear();
                    *self.buf = Some(
                        String::from_utf8(v)
                            .expect("the vec is empty, so its a valid empty string"),
                    );
                }
                Ok(StrOrNumber::Str(Cow::Owned(encoded)))
            }
        }
    }
}

fn encode_hex_into(bytes: &[u8], buf: &mut Option<String>) -> StrOrNumber<'static> {
    let encoded = hex::encode(bytes);

    match buf {
        None => {
            *buf = Some(encoded);
            StrOrNumber::Buffered
        }
        Some(_) => StrOrNumber::Str(Cow::Owned(encoded)),
    }
}
