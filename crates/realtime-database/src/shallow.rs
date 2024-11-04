use std::collections::BTreeSet;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::Deserialize;
use serde::de::{self, IntoDeserializer};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shallow<T> {
    inner: BTreeSet<T>,
}

impl<T> fmt::Debug for Shallow<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<T> Deref for Shallow<T> {
    type Target = BTreeSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Shallow<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> IntoIterator for Shallow<T> {
    type Item = T;
    type IntoIter = <BTreeSet<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'de, T> Deserialize<'de> for Shallow<T>
where
    T: Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ShallowVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}

pub struct ShallowStringVisitor;

impl<'de> de::Visitor<'de> for ShallowStringVisitor {
    type Value = Shallow<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map from strings to 'true'")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Shallow {
            inner: BTreeSet::new(),
        })
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Shallow {
            inner: BTreeSet::new(),
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut inner = BTreeSet::new();

        while let Some(key) = seq.next_element::<String>()? {
            inner.insert(key);
        }

        Ok(Shallow { inner })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut inner = BTreeSet::new();

        while let Some((key, _)) = map.next_entry::<String, de::IgnoredAny>()? {
            inner.insert(key);
        }

        Ok(Shallow { inner })
    }
}

pub struct ShallowVisitor<'de, T> {
    _marker: std::marker::PhantomData<(&'de (), T)>,
}

impl<'de, T> de::DeserializeSeed<'de> for ShallowVisitor<'de, T>
where
    T: Deserialize<'de> + Ord,
{
    type Value = Shallow<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, T> de::Visitor<'de> for ShallowVisitor<'de, T>
where
    T: Deserialize<'de> + Ord,
{
    type Value = Shallow<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map of string keys to boolean values")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut inner = BTreeSet::new();

        while let Some((key, _)) = map.next_entry::<T, de::IgnoredAny>()? {
            inner.insert(key);
        }

        Ok(Shallow { inner })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SingleKey<T> {
    key: Option<T>,
}

impl<T> Default for SingleKey<T> {
    fn default() -> Self {
        Self { key: None }
    }
}

impl<T> SingleKey<T> {
    pub(crate) fn into_inner(self) -> Option<T> {
        self.key
    }
}

impl<'de, T> de::Deserialize<'de> for SingleKey<T>
where
    T: de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(SingleKeyVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}

struct SingleKeyVisitor<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<'de, T> de::Visitor<'de> for SingleKeyVisitor<T>
where
    T: de::Deserialize<'de>,
{
    type Value = SingleKey<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a single key/value pair, where the value will be ignored")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let key = T::deserialize(v.into_deserializer())?;

        Ok(SingleKey { key: Some(key) })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let key = T::deserialize(v.into_deserializer())?;

        Ok(SingleKey { key: Some(key) })
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SingleKey { key: None })
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SingleKey { key: None })
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let inner: (T, _) = data.variant()?;

        Ok(SingleKey { key: Some(inner.0) })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let key = seq.next_element::<(T, de::IgnoredAny)>()?.map(|(k, _)| k);

        while seq.next_element::<de::IgnoredAny>()?.is_some() {
            // drain seq access
        }

        Ok(SingleKey { key })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let key = map.next_entry::<T, de::IgnoredAny>()?.map(|(k, _)| k);

        while map
            .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
            .is_some()
        {
            // drain the map access
        }

        Ok(SingleKey { key })
    }
}
