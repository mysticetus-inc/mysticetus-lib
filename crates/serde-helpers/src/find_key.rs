use std::fmt;
use std::marker::PhantomData;

use serde::de;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FindKey<S> {
    target_key: S,
}

impl<S> FindKey<S> {
    #[inline]
    pub const fn new(target_key: S) -> Self {
        Self { target_key }
    }

    pub const fn new_case_insensitive(target_key: S) -> FindKey<CaseInsensitive<S>> {
        FindKey {
            target_key: CaseInsensitive(target_key),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CaseInsensitive<S>(pub S);

pub trait Matches {
    fn matches(&self, found_key: &str) -> bool;
}

impl<T> Matches for T
where
    T: AsRef<str>,
{
    #[inline]
    fn matches(&self, found_key: &str) -> bool {
        self.as_ref().eq(found_key)
    }
}

impl<S> Matches for CaseInsensitive<S>
where
    S: AsRef<str>,
{
    #[inline]
    fn matches(&self, found_key: &str) -> bool {
        found_key.eq_ignore_ascii_case(self.0.as_ref())
    }
}

impl<S> From<S> for FindKey<S> {
    #[inline]
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyStatus {
    Found,
    NotFound,
}

impl KeyStatus {
    /// shorthand for 'ks == KeyStatus::Found'
    #[inline]
    pub const fn is_found(self) -> bool {
        matches!(self, Self::Found)
    }

    /// shorthand for 'ks == KeyStatus::NotFound'     
    #[inline]
    pub const fn is_not_found(self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl<'de, S> de::DeserializeSeed<'de> for FindKey<S>
where
    S: Matches + fmt::Display,
{
    type Value = KeyStatus;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(&self)
    }
}

impl<'de, S> de::DeserializeSeed<'de> for &FindKey<S>
where
    S: Matches + fmt::Display,
{
    type Value = KeyStatus;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de, S> de::Visitor<'de> for &FindKey<S>
where
    S: Matches + fmt::Display,
{
    type Value = KeyStatus;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a map key who's value is equal to '{}'",
            self.target_key
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if self.target_key.matches(v) {
            Ok(KeyStatus::Found)
        } else {
            Ok(KeyStatus::NotFound)
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match std::str::from_utf8(v) {
            Ok(s) => self.visit_str(s),
            Err(_) => Err(de::Error::invalid_value(
                de::Unexpected::Bytes(v),
                &FmtVisitor(
                    format_args!(
                        "expected a valid UTF-8 key that matches '{}'",
                        self.target_key
                    ),
                    PhantomData::<()>,
                ),
            )),
        }
    }
}

struct FmtVisitor<'a, T = ()>(fmt::Arguments<'a>, PhantomData<T>);

impl<'de, T> de::Visitor<'de> for FmtVisitor<'_, T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(self.0)
    }
}
