use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::string_dst::StringDst;
use serde::de::{self, Deserialize};
use serde::ser::{self, Impossible, Serialize, Serializer};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyCapture<D>(pub D);

impl<D> KeyCapture<D>
where
    D: Default,
{
    pub fn take(&mut self) -> D {
        std::mem::take(&mut self.0)
    }
}

impl<D> Deref for KeyCapture<D> {
    type Target = D;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DerefMut for KeyCapture<D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<D> From<D> for KeyCapture<D> {
    #[inline]
    fn from(inner: D) -> Self {
        Self(inner)
    }
}

impl<'de, T> de::DeserializeSeed<'de> for KeyCapture<T>
where
    T: StringDst,
{
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // make sure the key is empty and ready to go for each time we're called as a seed.
        // if we dont, keys end up getting appended over and over, and managing that in
        // complex custom deserialization loops is less than ideal.
        self.0.clear();
        deserializer.deserialize_any(self)
    }
}

impl<'de, D> de::Visitor<'de> for KeyCapture<D>
where
    D: StringDst,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a str/String key")
    }

    fn visit_str<E>(mut self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0.handle_str(v);
        Ok(())
    }

    fn visit_string<E>(mut self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0.handle_string(v);
        Ok(())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = std::str::from_utf8(v).map_err(de::Error::custom)?;
        self.visit_str(s)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = String::from_utf8(v).map_err(de::Error::custom)?;
        self.visit_string(s)
    }

    fn visit_some<De>(self, deserializer: De) -> Result<Self::Value, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        match Cow::deserialize(deserializer)? {
            Cow::Borrowed(b) => self.visit_str(b),
            Cow::Owned(o) => self.visit_string(o),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(())
    }

    fn visit_newtype_struct<De>(self, deserializer: De) -> Result<Self::Value, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        self.visit_some(deserializer)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        data.variant_seed(self)?;
        Ok(())
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut buf: [u8; 4] = [0; 4];
        self.visit_str(v.encode_utf8(&mut buf))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidKey {
    NotAString(de::Unexpected<'static>),
    InvalidUtf8(std::str::Utf8Error),
    Custom(Box<str>),
}

impl fmt::Display for InvalidKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAString(found) => {
                write!(formatter, "expected a string key, found ")?;
                found.fmt(formatter)
            }
            Self::InvalidUtf8(err) => err.fmt(formatter),
            Self::Custom(err) => err.fmt(formatter),
        }
    }
}

impl std::error::Error for InvalidKey {}

impl ser::Error for InvalidKey {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom(msg.to_string().into_boxed_str())
    }
}

macro_rules! impl_serialize_fns {
    ($($fn_name:ident($arg_ty:ty) -> $variant:ident),* $(,)?) => {
        $(
            fn $fn_name(self, v: $arg_ty) -> Result<Self::Ok, Self::Error> {
                Err(InvalidKey::NotAString(de::Unexpected::$variant(v as _)))
            }
        )*
    };
}

impl<D> Serializer for KeyCapture<D>
where
    D: StringDst,
{
    type Ok = ();
    type Error = InvalidKey;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    impl_serialize_fns! {
        serialize_i8(i8) -> Signed,
        serialize_i16(i16) -> Signed,
        serialize_i32(i32) -> Signed,
        serialize_i64(i64) -> Signed,
        serialize_u8(u8) -> Unsigned,
        serialize_u16(u16) -> Unsigned,
        serialize_u32(u32) -> Unsigned,
        serialize_u64(u64) -> Unsigned,
        serialize_bool(bool) -> Bool,
        serialize_char(char) -> Char,
        serialize_f64(f64) -> Float,
        serialize_f32(f32) -> Float,
    }

    serde::serde_if_integer128! {
        impl_serialize_fns! {
            serialize_i128(i128) -> Signed,
            serialize_u128(u128) -> Unsigned,
        }
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.handle_str(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.0.handle_static_str(name);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = std::str::from_utf8(v).map_err(InvalidKey::InvalidUtf8)?;
        self.serialize_str(s)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::Seq))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::Map))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::Other("tuple")))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::Other(name)))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::Other(name)))
    }

    fn serialize_unit_variant(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_static_str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(InvalidKey::NotAString(de::Unexpected::NewtypeStruct))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::TupleVariant))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(InvalidKey::NotAString(de::Unexpected::StructVariant))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(InvalidKey::NotAString(de::Unexpected::NewtypeVariant))
    }
}
