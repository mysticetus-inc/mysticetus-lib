use std::borrow::Cow;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed};

pub struct TryBorrowVisitor<'a, T: ?Sized>(PhantomData<fn(&'a T)>);

impl<'a, 'de: 'a> de::Visitor<'de> for TryBorrowVisitor<'a, [u8]> {
    type Value = Cow<'a, [u8]>;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("bytes")
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Borrowed(v))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v.to_vec()))
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v))
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Borrowed(v.as_bytes()))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v.as_bytes().to_vec()))
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v.into_bytes()))
    }
}

impl<'a, 'de: 'a> de::Visitor<'de> for TryBorrowVisitor<'a, str> {
    type Value = Cow<'a, str>;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match std::str::from_utf8(v) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(err) => Err(E::invalid_value(
                de::Unexpected::Bytes(v),
                &err.to_string().as_str(),
            )),
        }
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match std::str::from_utf8(v) {
            Ok(s) => Ok(Cow::Owned(s.to_owned())),
            Err(err) => Err(E::invalid_value(
                de::Unexpected::Bytes(v),
                &err.to_string().as_str(),
            )),
        }
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(Cow::Owned(s)),
            Err(err) => Err(E::invalid_value(
                de::Unexpected::Bytes(err.as_bytes()),
                &err.to_string().as_str(),
            )),
        }
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Borrowed(v))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v.to_owned()))
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v))
    }
}

impl<'a, 'de: 'a, T> DeserializeSeed<'de> for TryBorrowVisitor<'a, T>
where
    T: ToOwned + ?Sized + 'static,
    Self: de::Visitor<'de>,
{
    type Value = <Self as de::Visitor<'de>>::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<str>() {
            deserializer.deserialize_string(TryBorrowVisitor(PhantomData))
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<[u8]>() {
            deserializer.deserialize_byte_buf(TryBorrowVisitor(PhantomData))
        } else {
            unreachable!("TryBorrowVisitor only has visitor impls for str and [u8]")
        }
    }
}

#[inline]
pub fn serialize<T, S>(value: &Cow<'_, T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: serde::Serialize + ToOwned + ?Sized,
    S: serde::Serializer,
{
    T::serialize(value, serializer)
}

#[inline]
pub fn deserialize<'a, 'de: 'a, T, D>(deserializer: D) -> Result<Cow<'a, T>, D::Error>
where
    T: ?Sized + ToOwned + 'static,
    D: serde::Deserializer<'de>,
    TryBorrowVisitor<'de, T>: de::Visitor<'de, Value = Cow<'a, T>>,
{
    TryBorrowVisitor(PhantomData).deserialize(deserializer)
}

pub mod optional {
    use std::borrow::Cow;
    use std::marker::PhantomData;

    use serde::de::DeserializeSeed;

    use super::TryBorrowVisitor;

    #[inline]
    pub fn serialize<T, S>(value: &Option<Cow<'_, T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: serde::Serialize + ToOwned + ?Sized,
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&value.as_deref(), serializer)
    }

    #[inline]
    pub fn deserialize<'a, 'de: 'a, T, D>(deserializer: D) -> Result<Option<Cow<'a, T>>, D::Error>
    where
        T: ?Sized + ToOwned + 'static,
        TryBorrowVisitor<'de, T>: serde::de::Visitor<'de, Value = Cow<'a, T>>,
        D: serde::Deserializer<'de>,
    {
        crate::optional_visitor::OptionalVisitor(TryBorrowVisitor(PhantomData))
            .deserialize(deserializer)
    }
}
