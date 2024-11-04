use std::fmt;
use std::marker::PhantomData;

use serde::de;

use crate::kvp::Kvp;

pub struct MatchKvp<'a, Key, Value, Target = Key> {
    target: &'a Target,
    _marker: PhantomData<(Key, Value)>,
}

impl<'a, K, V, T> MatchKvp<'a, K, V, T> {
    pub const fn new(target: &'a T) -> Self {
        Self {
            target,
            _marker: PhantomData,
        }
    }
}

impl<'de, K, V, T> de::DeserializeSeed<'de> for MatchKvp<'_, K, V, T>
where
    K: de::Deserialize<'de>,
    V: de::Deserialize<'de>,
    K: PartialEq<T>,
    T: fmt::Debug,
{
    type Value = Kvp<K, V>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, K, V, T> de::Visitor<'de> for MatchKvp<'_, K, V, T>
where
    K: de::Deserialize<'de>,
    V: de::Deserialize<'de>,
    K: PartialEq<T>,
    T: fmt::Debug,
{
    type Value = Kvp<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a single key value pair, where the key matches '{:?}'",
            self.target
        )
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<K>()? {
            if key.eq(self.target) {
                let value = map.next_value::<V>()?;

                // drain the map once we find the key/value pair
                while map
                    .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
                    .is_some()
                {}

                return Ok(Kvp { key, value });
            } else {
                // ignore if the key isnt a match
                map.next_value::<de::IgnoredAny>()?;
            }
        }

        Err(de::Error::custom(format!(
            "no key found that matches '{:?}'",
            self.target
        )))
    }
}
