use core::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FromStrVisitor<T> {
    _marker: PhantomData<fn(T)>,
}

impl<T> FromStrVisitor<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> FromStrVisitor<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(Self::new())
    }

    #[inline]
    fn try_parse<E>(self, v: &str) -> Result<T, E>
    where
        E: de::Error,
        T: FromStr,
        T::Err: fmt::Display,
    {
        use serde::de::Unexpected::Str;

        v.parse()
            .map_err(|err| de::Error::invalid_value(Str(v), &FromStrError(self, err)))
    }
}

impl<T> Default for FromStrVisitor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T> de::DeserializeSeed<'de> for FromStrVisitor<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de, T> de::Visitor<'de> for FromStrVisitor<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid string representation of ")?;
        formatter.write_str(std::any::type_name::<T>())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = std::str::from_utf8(v)
            .map_err(|_| E::invalid_value(de::Unexpected::Bytes(v), &self))?;
        self.visit_str(s)
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.try_parse(v)
    }
}

struct FromStrError<T: FromStr>(FromStrVisitor<T>, T::Err);

impl<T: FromStr> de::Expected for FromStrError<T>
where
    T::Err: fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.1, formatter)?;
        formatter.write_str(": ")?;
        de::Visitor::expecting(&self.0, formatter)
    }
}
