//! [`Kvp`], a Serialiable type representing a single key/value pair.
//!
//! Useful in scenerios where defining a 1 field struct doesn't work, i.e
//! needing to use a dynamic value as the key.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeMap, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kvp<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> Serialize for Kvp<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.key, &self.value)?;
        map.end()
    }
}

impl<'de, K, V> Deserialize<'de> for Kvp<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KvpVisitor(PhantomData))
    }
}

struct KvpVisitor<K, V>(PhantomData<(K, V)>);

impl<'de, K, V> de::Visitor<'de> for KvpVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = Kvp<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a single key value pair")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let kvp = match map.next_entry()? {
            Some((key, value)) => Kvp { key, value },
            None => return Err(de::Error::custom("expected at least 1 Kvp entry")),
        };

        // most deserializers get mad if you dont drain all entries, so do that before returning
        while map
            .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
            .is_some()
        {}

        Ok(kvp)
    }
}
